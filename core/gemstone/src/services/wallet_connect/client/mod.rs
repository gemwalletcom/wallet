mod model;
mod port;
mod relay;
mod state;
#[cfg(test)]
pub(crate) mod testkit;

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use chrono::Utc;
use futures::future::{Either, select};
use gem_crypto::random;
use gem_encoding::encode_hex;
use primitives::{Account, ApplicationMetadata, Platform, Wallet, WalletConnectLink, WalletConnection, WalletConnectionSession};
use serde::de::DeserializeOwned;
use serde_json::value::RawValue;
use wallet_connect_protocol::authenticate::{self, AuthPayload, AuthenticateParams, AuthenticateResponse, Issuer};
use wallet_connect_protocol::crypto::{self, Key};
use wallet_connect_protocol::relay::rpc::{self, ApproveSession};
use wallet_connect_protocol::relay::{RelayFrame, RelayMessage, parse_frame};
use wallet_connect_protocol::sign::method::{
    METHOD_PAIRING_DELETE, METHOD_PAIRING_PING, METHOD_SESSION_AUTHENTICATE, METHOD_SESSION_DELETE, METHOD_SESSION_PING, METHOD_SESSION_PROPOSE, METHOD_SESSION_REQUEST, METHOD_SESSION_SETTLE, METHOD_SESSION_UPDATE,
};
use wallet_connect_protocol::sign::namespaces::{self, Supported};
use wallet_connect_protocol::sign::payload::{self, Payload};
use wallet_connect_protocol::sign::types::Redirect;
use wallet_connect_protocol::sign::{Metadata, Participant, ProposeResponse, Publish, Reason, Relay, SessionProposal, SessionRequest, SettleParams, UpdateParams};
use wallet_connect_protocol::verify::{self, Attestation, Verification, VerifyPublicKey};
use wallet_connect_protocol::{PairingUri, ProtocolError, envelope};

use super::model::{GemWalletConnectFailure, GemWalletConnectOutcome, GemWalletConnectRejection, GemWalletConnectRejectionReason, GemWalletConnectResponse, GemWalletConnectRpcError, GemWalletConnectSessionRequest};
use super::{GemWalletConnectService, rules};
use crate::alien::{AlienHttpMethod, AlienProvider, AlienTarget};
use crate::config::wallet_connect::get_wallet_connect_config;
use crate::services::clock::sleep;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemSecureStore;
use gem_wallet_connect::{WalletConnectResponseType, validate_domain};
use state::{ClientState, Outgoing, Pairing, PendingRequest, Session, Topic};

pub use model::{GemWalletConnectAuthMessage, GemWalletConnectAuthentication, GemWalletConnectEvent, GemWalletConnectProposal, GemWalletConnectRelay};
pub use port::{GemWalletConnectListener, GemWalletConnectTransport};
pub use relay::wallet_connect_relay;

const STATE_KEY: &str = "wallet_connect_client";
const SESSION_LIFETIME_SECONDS: u64 = 7 * 86_400;
const PAIRING_LIFETIME_SECONDS: u64 = 300;
const APPROVAL_TTL_SECONDS: u64 = 300;
const EXPIRY_CHECK_INTERVAL: Duration = Duration::from_secs(3);
const VERIFY_TIMEOUT: Duration = Duration::from_secs(5);
const HTTP_OK: u16 = 200;

#[derive(uniffi::Object)]
pub struct GemWalletConnectClient {
    service: Arc<GemWalletConnectService>,
    store: Arc<dyn GemSecureStore>,
    provider: Arc<dyn AlienProvider>,
    transport: Arc<dyn GemWalletConnectTransport>,
    listener: Arc<dyn GemWalletConnectListener>,
    platform: Platform,
    state: Mutex<Option<ClientState>>,
    pairing_requests: Mutex<HashMap<String, PairingRequest>>,
    linked_topics: Mutex<HashSet<String>>,
}

#[derive(Clone)]
enum PairingRequest {
    Proposal(PendingProposal),
    Authentication(PendingAuthentication),
}

#[derive(Clone)]
struct PendingProposal {
    request_id: u64,
    pairing_topic: String,
    proposal: SessionProposal,
    verification: Verification,
}

#[derive(Clone)]
struct PendingAuthentication {
    request_id: u64,
    pairing_topic: String,
    params: AuthenticateParams,
    verification: Verification,
}

impl PairingRequest {
    fn id(&self) -> String {
        match self {
            Self::Proposal(pending) => pending.proposal.proposer.public_key.clone(),
            Self::Authentication(pending) => pending.request_id.to_string(),
        }
    }

    fn pairing_topic(&self) -> &str {
        match self {
            Self::Proposal(pending) => &pending.pairing_topic,
            Self::Authentication(pending) => &pending.pairing_topic,
        }
    }
}

#[uniffi::export]
impl GemWalletConnectClient {
    #[uniffi::constructor]
    pub fn new(
        service: Arc<GemWalletConnectService>,
        store: Arc<dyn GemSecureStore>,
        provider: Arc<dyn AlienProvider>,
        transport: Arc<dyn GemWalletConnectTransport>,
        listener: Arc<dyn GemWalletConnectListener>,
        platform: Platform,
    ) -> Self {
        Self {
            service,
            store,
            provider,
            transport,
            listener,
            platform,
            state: Mutex::new(None),
            pairing_requests: Mutex::new(HashMap::new()),
            linked_topics: Mutex::new(HashSet::new()),
        }
    }

    pub fn link_opened(&self, link: WalletConnectLink) {
        let topic = match link {
            WalletConnectLink::Connect { uri } => PairingUri::parse(&uri, now()).ok().map(|uri| uri.topic),
            WalletConnectLink::Session { topic } => Some(topic),
            WalletConnectLink::Request => None,
        };
        if let Some(topic) = topic {
            lock(&self.linked_topics).insert(topic);
        }
    }

    pub fn needs_connection(&self) -> Result<bool, GemServiceError> {
        self.read(|state| !state.topics().is_empty() || !state.outbox.is_empty())
    }

    pub async fn connected(&self) -> Result<(), GemServiceError> {
        let now = now();
        let (topics, frames) = self.update_if_changed(|state| {
            let changed = state.prune(now);
            ((state.topics(), state.outbox.iter().map(|outgoing| outgoing.frame.clone()).collect::<Vec<_>>()), changed)
        })?;
        if !topics.is_empty() {
            self.transport.send(rpc::batch_subscribe(message_id(), &topics)).await?;
        }
        for frame in frames {
            self.transport.send(frame).await?;
        }
        Ok(())
    }

    pub async fn restore(&self) -> Result<(), GemServiceError> {
        let now = now();
        self.update_if_changed(|state| ((), state.prune(now)))?;
        self.sync_connections().await?;
        self.show_next_request()
    }

    pub async fn receive(&self, frame: String) -> Result<(), GemServiceError> {
        match parse_frame(&frame)? {
            RelayFrame::Message { id, message } => {
                let received = self.receive_message(message).await;
                if !matches!(received, Err(GemServiceError::Store { .. } | GemServiceError::Platform { .. })) {
                    self.transport.send(rpc::acknowledge(&id)).await?;
                }
                received
            }
            RelayFrame::Accepted { id: Some(id) } => self.delivered(id),
            RelayFrame::Accepted { id: None } | RelayFrame::Refused => Ok(()),
        }
    }

    pub async fn pair(&self, uri: String) -> Result<(), GemServiceError> {
        let now = now();
        let uri = PairingUri::parse(&uri, now)?;
        let pairing = Pairing {
            topic: uri.topic.clone(),
            key: encode_hex(uri.key.as_slice()),
            expiry: uri.expiry.unwrap_or(now + PAIRING_LIFETIME_SECONDS),
            authenticates: uri.authenticates,
        };
        let is_new = self.update(|state| {
            let is_new = state.topic(&pairing.topic).is_none();
            if is_new {
                state.pairings.push(pairing.clone());
            }
            is_new
        })?;
        if !is_new && self.is_answering(&pairing.topic) {
            return Ok(());
        }
        if let Err(error) = self.transport.send(rpc::subscribe(message_id(), &pairing.topic)).await {
            if is_new {
                self.update(|state| state.remove_pairing(&pairing.topic))?;
            }
            return Err(error);
        }
        Ok(())
    }

    pub async fn approve_session(&self, proposal_id: String, wallet: Wallet) -> Result<(), GemServiceError> {
        let Some(PairingRequest::Proposal(pending)) = self.pairing_request(&proposal_id) else {
            return Err(not_found("proposal"));
        };
        if rules::is_expired(pending.proposal.expiry_timestamp, Utc::now()) {
            return self.expire(&proposal_id, &pending.pairing_topic);
        }
        if self
            .service
            .is_origin_rejected(pending.proposal.proposer.metadata.url.clone(), pending.verification.origin.clone(), pending.verification.status.clone())
        {
            if self.take_pairing_request(&proposal_id).is_some() {
                self.reject_proposal(&pending, GemWalletConnectRejectionReason::UserRejected).await?;
            }
            return Err(invalid_origin());
        }
        let approval = self.service.session_approval(wallet.clone());
        let chains: Vec<String> = approval.chains.iter().flat_map(|chain| rules::caip2_chain_ids(*chain)).collect();
        let accounts: Vec<String> = approval
            .accounts
            .iter()
            .flat_map(|account| rules::caip2_chain_ids(account.chain).into_iter().map(|chain_id| format!("{chain_id}:{}", account.address)))
            .collect();
        let supported = Supported {
            chains: &chains,
            accounts: &accounts,
            methods: &approval.methods,
            events: &approval.events,
        };
        let namespaces = match namespaces::approved_namespaces(&pending.proposal, &supported) {
            Ok(namespaces) => namespaces,
            Err(error) => {
                let reason = rules::namespace_rejection(error);
                if self.take_pairing_request(&proposal_id).is_some() {
                    self.reject_proposal(&pending, reason).await?;
                }
                return Err(GemServiceError::Unsupported {
                    msg: self.service.session_rejection(reason).message,
                });
            }
        };
        let approved_chains = namespaces::approved_chains(&namespaces);
        let session_properties = self
            .service
            .config_session_properties(pending.proposal.session_properties.clone().unwrap_or_default(), approved_chains.clone(), approval.accounts)
            .await;
        let pairing_key = self.pairing_key(&pending.pairing_topic)?;
        let secret = crypto::generate_key()?;
        let public_key = encode_hex(crypto::public_key(&secret));
        let proposer_key = crypto::parse_key(&pending.proposal.proposer.public_key)?;
        let session_key = crypto::shared_key(&secret, &proposer_key)?;
        let session_topic = crypto::topic(session_key.as_slice());
        let expiry = now() + SESSION_LIFETIME_SECONDS;
        let response = payload::success(
            pending.request_id,
            &ProposeResponse {
                relay: Relay::irn(),
                responder_public_key: public_key.clone(),
            },
        );
        let settle = payload::request(
            message_id(),
            METHOD_SESSION_SETTLE,
            SettleParams {
                relay: Relay::irn(),
                controller: Participant { public_key, metadata: self.metadata() },
                namespaces: namespaces.clone(),
                session_properties,
                expiry,
            },
        );
        let frame_id = message_id();
        let frame = rpc::approve_session(
            frame_id,
            ApproveSession {
                pairing_topic: &pending.pairing_topic,
                session_topic: &session_topic,
                session_proposal_response: &envelope::encrypt(&pairing_key, &response)?,
                session_settlement_request: &envelope::encrypt(&session_key, &settle)?,
                correlation_id: pending.request_id,
                approved_chains,
                approved_methods: namespaces::approved_methods(&namespaces),
                approved_events: namespaces::approved_events(&namespaces),
            },
        );
        let approval = Outgoing {
            id: frame_id,
            frame,
            expiry: now() + APPROVAL_TTL_SECONDS,
        };
        if self.take_pairing_request(&proposal_id).is_none() {
            return Err(not_found("proposal"));
        }
        let session = Session {
            topic: session_topic,
            key: encode_hex(session_key.as_slice()),
            peer: pending.proposal.proposer.metadata,
            namespaces,
            expiry,
            origin: pending.verification.origin,
            validation: pending.verification.status,
        };
        let peer = session.peer.clone();
        self.store_session(session, wallet, &pending.pairing_topic, approval).await?;
        self.return_to_dapp(&pending.pairing_topic, &peer);
        Ok(())
    }

    pub async fn reject_session(&self, proposal_id: String, reason: GemWalletConnectRejectionReason) -> Result<(), GemServiceError> {
        let Some(PairingRequest::Proposal(pending)) = self.take_pairing_request(&proposal_id) else {
            return Ok(());
        };
        self.reject_proposal(&pending, reason).await?;
        self.return_to_dapp(&pending.pairing_topic, &pending.proposal.proposer.metadata);
        Ok(())
    }

    pub async fn answer_request(&self, topic: String, request_id: String) -> Result<Option<GemWalletConnectFailure>, GemServiceError> {
        let id = parse_request_id(&request_id)?;
        let Some((request, session)) = self.read(|state| state.request(&topic, id))? else {
            return Ok(None);
        };
        let outcome = self.request_outcome(&request).await;
        let (response, failure) = match outcome {
            GemWalletConnectOutcome::Ignored => return Ok(None),
            GemWalletConnectOutcome::Respond { response } => (response, None),
            GemWalletConnectOutcome::Fail { error, failure } => (GemWalletConnectResponse::Error { error }, Some(failure)),
            GemWalletConnectOutcome::AddSessionAccount { account } => {
                self.add_session_account(&session, &account).await?;
                (GemWalletConnectResponse::Null, None)
            }
        };
        self.respond(&request, &session, response).await?;
        match failure {
            None => self.return_after_last_request(&session)?,
            Some(_) => {
                self.forget_link(&session.topic);
            }
        }
        Ok(failure)
    }

    pub async fn reject_request(&self, topic: String, request_id: String) -> Result<(), GemServiceError> {
        let id = parse_request_id(&request_id)?;
        let Some((request, session)) = self.read(|state| state.request(&topic, id))? else {
            return Ok(());
        };
        self.respond(&request, &session, GemWalletConnectResponse::Error { error: rules::user_rejected_error() }).await?;
        self.return_after_last_request(&session)
    }

    pub async fn disconnect(&self, topic: String) -> Result<(), GemServiceError> {
        self.service.delete_session(topic.clone()).await?;
        let Some((session, dropped)) = self.update(|state| state.remove_session(&topic))? else {
            return Ok(());
        };
        self.forget_link(&topic);
        self.close_requests(&dropped);
        let request = payload::request(message_id(), METHOD_SESSION_DELETE, Reason::user_disconnected());
        self.publish(&session.topic, &crypto::parse_key(&session.key)?, &request, Publish::SessionDelete).await
    }

    pub fn authentication_message(&self, authentication_id: String, wallet: Wallet) -> Result<GemWalletConnectAuthMessage, GemServiceError> {
        let Some(PairingRequest::Authentication(pending)) = self.pairing_request(&authentication_id) else {
            return Err(not_found("authentication"));
        };
        let (account, issuer, payload) = self.authentication_payload(&pending, wallet)?;
        Ok(GemWalletConnectAuthMessage {
            account,
            message: authenticate::message(&payload, &issuer)?,
        })
    }

    pub async fn approve_authentication(&self, authentication_id: String, wallet: Wallet, signature: String) -> Result<(), GemServiceError> {
        let Some(PairingRequest::Authentication(pending)) = self.pairing_request(&authentication_id) else {
            return Err(not_found("authentication"));
        };
        if rules::is_expired(pending.params.expiry_timestamp, Utc::now()) {
            return self.expire(&authentication_id, &pending.pairing_topic);
        }
        let (_, issuer, payload) = self.authentication_payload(&pending, wallet.clone())?;
        let secret = crypto::generate_key()?;
        let requester_key = crypto::parse_key(&pending.params.requester.public_key)?;
        let session_key = crypto::shared_key(&secret, &requester_key)?;
        let response = payload::success(
            pending.request_id,
            &AuthenticateResponse {
                responder: Participant {
                    public_key: encode_hex(crypto::public_key(&secret)),
                    metadata: self.metadata(),
                },
                cacaos: vec![authenticate::cacao(&payload, &issuer, signature)],
            },
        );
        let approval = authentication_answer(&pending, &secret, &response, Publish::AuthenticationApproval)?;
        if self.take_pairing_request(&authentication_id).is_none() {
            return Err(not_found("authentication"));
        }
        let requester = pending.params.requester.metadata.clone();
        let Some(namespaces) = authenticate::session_namespaces(&payload, &issuer) else {
            self.store_and_send(approval, |state| state.remove_pairing(&pending.pairing_topic)).await?;
            self.return_to_dapp(&pending.pairing_topic, &requester);
            return Ok(());
        };
        let session = Session {
            topic: crypto::topic(session_key.as_slice()),
            key: encode_hex(session_key.as_slice()),
            peer: pending.params.requester.metadata,
            namespaces,
            expiry: now() + SESSION_LIFETIME_SECONDS,
            origin: pending.verification.origin,
            validation: pending.verification.status,
        };
        let subscription = rpc::subscribe(message_id(), &session.topic);
        self.store_session(session, wallet, &pending.pairing_topic, approval).await?;
        self.send_frame(subscription).await;
        self.return_to_dapp(&pending.pairing_topic, &requester);
        Ok(())
    }

    pub async fn reject_authentication(&self, authentication_id: String) -> Result<(), GemServiceError> {
        let Some(PairingRequest::Authentication(pending)) = self.take_pairing_request(&authentication_id) else {
            return Ok(());
        };
        let response = rejection(pending.request_id, &self.service.session_rejection(GemWalletConnectRejectionReason::UserRejected));
        let rejection = authentication_answer(&pending, &crypto::generate_key()?, &response, Publish::AuthenticationRejection)?;
        self.store_and_send(rejection, |state| state.remove_pairing(&pending.pairing_topic)).await?;
        self.return_to_dapp(&pending.pairing_topic, &pending.params.requester.metadata);
        Ok(())
    }
}

impl GemWalletConnectClient {
    async fn receive_message(&self, message: RelayMessage) -> Result<(), GemServiceError> {
        let Some(topic) = self.read(|state| state.topic(&message.topic))? else {
            return Ok(());
        };
        let key = crypto::parse_key(match &topic {
            Topic::Pairing(pairing) => &pairing.key,
            Topic::Session(session) => &session.key,
        })?;
        let decrypted = envelope::decrypt(&key, &message.message)?;
        let Payload::Request { id, method, params } = Payload::parse(&decrypted)? else {
            return Ok(());
        };
        match (topic, method.as_str()) {
            (Topic::Pairing(pairing), METHOD_SESSION_PROPOSE) => self.receive_proposal(pairing, &message, &decrypted, id, decode(&params)?).await,
            (Topic::Pairing(pairing), METHOD_SESSION_AUTHENTICATE) => self.receive_authentication(pairing, &message, &decrypted, id, decode(&params)?).await,
            (Topic::Pairing(pairing), METHOD_PAIRING_PING) => self.answer_ping(&pairing.topic, &key, id, Publish::PairingPingResponse).await,
            (Topic::Pairing(pairing), METHOD_PAIRING_DELETE) => {
                lock(&self.pairing_requests).remove(&pairing.topic);
                let response = outgoing_publish(&pairing.topic, &envelope::encrypt(&key, &payload::success(id, &true))?, Publish::PairingDeleteResponse);
                self.store_and_send(response, |state| state.remove_pairing(&pairing.topic)).await
            }
            (Topic::Session(session), METHOD_SESSION_REQUEST) => self.receive_request(session, &message, &key, id, decode(&params)?).await,
            (Topic::Session(session), METHOD_SESSION_PING) => self.answer_ping(&session.topic, &key, id, Publish::SessionPingResponse).await,
            (Topic::Session(session), METHOD_SESSION_DELETE) => self.receive_session_delete(session, &key, id).await,
            _ => Ok(()),
        }
    }

    async fn receive_proposal(&self, pairing: Pairing, message: &RelayMessage, decrypted: &[u8], request_id: u64, proposal: SessionProposal) -> Result<(), GemServiceError> {
        if (pairing.authenticates && rules::supports_authentication(self.platform)) || rules::is_expired(proposal.expiry_timestamp, Utc::now()) || self.is_answering(&pairing.topic) {
            return Ok(());
        }
        let verification = self.verification(message, decrypted).await;
        let event = GemWalletConnectProposal {
            id: proposal.proposer.public_key.clone(),
            pairing_topic: pairing.topic.clone(),
            metadata: application_metadata(&proposal.proposer.metadata),
            required_chain_ids: proposal.required_chain_ids(),
            optional_chain_ids: proposal.optional_chain_ids(),
            origin: verification.origin.clone(),
            validation: verification.status.clone(),
        };
        let pending = PendingProposal {
            request_id,
            pairing_topic: pairing.topic,
            proposal,
            verification,
        };
        if self.add_pairing_request(PairingRequest::Proposal(pending)) {
            self.listener.on_event(GemWalletConnectEvent::Proposal { proposal: event });
        }
        Ok(())
    }

    async fn receive_authentication(&self, pairing: Pairing, message: &RelayMessage, decrypted: &[u8], request_id: u64, params: AuthenticateParams) -> Result<(), GemServiceError> {
        if !rules::supports_authentication(self.platform) || rules::is_expired(params.expiry_timestamp, Utc::now()) || self.is_answering(&pairing.topic) {
            return Ok(());
        }
        let verification = self.verification(message, decrypted).await;
        let event = GemWalletConnectAuthentication {
            id: request_id.to_string(),
            metadata: application_metadata(&params.requester.metadata),
            chain_ids: params.auth_payload.chains.clone(),
            domain: params.auth_payload.domain.clone(),
            origin: verification.origin.clone(),
            validation: verification.status.clone(),
        };
        let pending = PendingAuthentication {
            request_id,
            pairing_topic: pairing.topic,
            params,
            verification,
        };
        if self.add_pairing_request(PairingRequest::Authentication(pending)) {
            self.listener.on_event(GemWalletConnectEvent::Authentication { authentication: event });
        }
        Ok(())
    }

    async fn receive_request(&self, session: Session, message: &RelayMessage, key: &Key, id: u64, request: SessionRequest) -> Result<(), GemServiceError> {
        if rules::is_expired(request.request.expiry_timestamp, Utc::now()) {
            return self.publish(&session.topic, key, &payload::failure(id, &reason(rules::request_expired_error())), Publish::SessionRequestResponse).await;
        }
        if let Err(refusal) = namespaces::authorization(&session.namespaces, &request.chain_id, &request.request.method) {
            return self.publish(&session.topic, key, &payload::failure(id, &refusal), Publish::SessionRequestResponse).await;
        }
        let verification = self.request_verification(message, &session);
        let pending = PendingRequest {
            topic: session.topic.clone(),
            id,
            method: request.request.method,
            params: request.request.params.get().to_string(),
            chain_id: request.chain_id,
            expiry: request.request.expiry_timestamp,
            origin: verification.origin,
            validation: verification.status,
        };
        let is_new = self.update_if_changed(|state| {
            let is_new = state.add_request(pending.clone());
            (is_new, is_new)
        })?;
        if is_new {
            self.listener.on_event(GemWalletConnectEvent::Request { request: session_request(&pending) });
        }
        Ok(())
    }

    async fn receive_session_delete(&self, session: Session, key: &Key, id: u64) -> Result<(), GemServiceError> {
        self.forget_link(&session.topic);
        if let Some((_, dropped)) = self.update(|state| state.remove_session(&session.topic))? {
            self.close_requests(&dropped);
        }
        self.service.delete_session(session.topic.clone()).await?;
        self.listener.on_event(GemWalletConnectEvent::SessionDeleted { topic: session.topic.clone() });
        self.publish(&session.topic, key, &payload::success(id, &true), Publish::SessionDeleteResponse).await
    }

    async fn request_outcome(&self, request: &PendingRequest) -> GemWalletConnectOutcome {
        let outcome = Box::pin(self.service.request_outcome(session_request(request)));
        let Some(expiry) = request.expiry else {
            return outcome.await;
        };
        match select(outcome, Box::pin(wait_until(expiry))).await {
            Either::Left((outcome, _)) => outcome,
            Either::Right((_, outcome)) => {
                self.listener.on_event(GemWalletConnectEvent::RequestExpired {
                    topic: request.topic.clone(),
                    request_id: request.id.to_string(),
                });
                outcome.await
            }
        }
    }

    async fn respond(&self, request: &PendingRequest, session: &Session, response: GemWalletConnectResponse) -> Result<(), GemServiceError> {
        let message = match response {
            GemWalletConnectResponse::Response { value } => payload::success(request.id, &response_value(value)?),
            GemWalletConnectResponse::Null => payload::success(request.id, &()),
            GemWalletConnectResponse::Error { error } => payload::failure(request.id, &reason(error)),
        };
        let key = crypto::parse_key(&session.key)?;
        let answer = outgoing_publish(&session.topic, &envelope::encrypt(&key, &message)?, Publish::SessionRequestResponse);
        let now = now();
        let answered = self.update_if_changed(|state| {
            let answered = state.answer(&request.topic, request.id, answer.clone(), now);
            (answered, answered)
        })?;
        if answered {
            self.send_frame(answer.frame).await;
        }
        self.show_next_request()
    }

    async fn add_session_account(&self, session: &Session, account: &str) -> Result<(), GemServiceError> {
        let namespaces = self
            .update(|state| {
                let stored = state.sessions.iter_mut().find(|stored| stored.topic == session.topic)?;
                stored.namespaces = namespaces::adding_account(&stored.namespaces, account);
                Some(stored.namespaces.clone())
            })?
            .ok_or_else(|| not_found("session"))?;
        let update = payload::request(message_id(), METHOD_SESSION_UPDATE, UpdateParams { namespaces: &namespaces });
        self.publish(&session.topic, &crypto::parse_key(&session.key)?, &update, Publish::SessionUpdate).await?;
        self.sync_connections().await
    }

    fn show_next_request(&self) -> Result<(), GemServiceError> {
        if let Some(request) = self.update_in_memory(ClientState::show_next_request)? {
            self.listener.on_event(GemWalletConnectEvent::Request { request: session_request(&request) });
        }
        Ok(())
    }

    fn return_after_last_request(&self, session: &Session) -> Result<(), GemServiceError> {
        if !self.read(|state| state.requests.iter().any(|request| request.topic == session.topic))? {
            self.return_to_dapp(&session.topic, &session.peer);
        }
        Ok(())
    }

    fn forget_link(&self, topic: &str) -> bool {
        lock(&self.linked_topics).remove(topic)
    }

    fn return_to_dapp(&self, topic: &str, dapp: &Metadata) {
        if !self.forget_link(topic) {
            return;
        }
        self.listener.on_event(GemWalletConnectEvent::ReturnToDapp {
            name: application_metadata(dapp).name,
            redirect: rules::dapp_redirect(dapp.redirect.as_ref()),
        });
    }

    fn delivered(&self, id: u64) -> Result<(), GemServiceError> {
        self.update_in_memory(|state| state.outbox.retain(|outgoing| outgoing.id != id))
    }

    fn pairing_request(&self, id: &str) -> Option<PairingRequest> {
        lock(&self.pairing_requests).values().find(|request| request.id() == id).cloned()
    }

    fn add_pairing_request(&self, request: PairingRequest) -> bool {
        match lock(&self.pairing_requests).entry(request.pairing_topic().to_string()) {
            Entry::Occupied(_) => false,
            Entry::Vacant(entry) => {
                entry.insert(request);
                true
            }
        }
    }

    fn take_pairing_request(&self, id: &str) -> Option<PairingRequest> {
        let mut requests = lock(&self.pairing_requests);
        let pairing_topic = requests.iter().find(|(_, request)| request.id() == id)?.0.clone();
        requests.remove(&pairing_topic)
    }

    fn is_answering(&self, pairing_topic: &str) -> bool {
        lock(&self.pairing_requests).contains_key(pairing_topic)
    }

    fn expire(&self, id: &str, pairing_topic: &str) -> Result<(), GemServiceError> {
        self.take_pairing_request(id);
        self.update(|state| state.remove_pairing(pairing_topic))?;
        Err(GemServiceError::Expired)
    }

    async fn reject_proposal(&self, pending: &PendingProposal, reason: GemWalletConnectRejectionReason) -> Result<(), GemServiceError> {
        let session_rejection = self.service.session_rejection(reason);
        let key = self.pairing_key(&pending.pairing_topic)?;
        let response = rejection(pending.request_id, &session_rejection);
        let answer = outgoing_publish(&pending.pairing_topic, &envelope::encrypt(&key, &response)?, Publish::ProposalRejection);
        self.store_and_send(answer, |state| state.remove_pairing(&pending.pairing_topic)).await?;
        if session_rejection.deletes_session {
            self.service.delete_session(pending.pairing_topic.clone()).await?;
        }
        Ok(())
    }

    async fn answer_ping(&self, topic: &str, key: &Key, id: u64, method: Publish) -> Result<(), GemServiceError> {
        let response = outgoing_publish(topic, &envelope::encrypt(key, &payload::success(id, &true))?, method);
        self.send_frame(response.frame).await;
        Ok(())
    }

    fn close_requests(&self, requests: &[PendingRequest]) {
        for request in requests {
            self.listener.on_event(GemWalletConnectEvent::RequestExpired {
                topic: request.topic.clone(),
                request_id: request.id.to_string(),
            });
        }
    }

    async fn publish(&self, topic: &str, key: &Key, message: &[u8], method: Publish) -> Result<(), GemServiceError> {
        self.store_and_send(outgoing_publish(topic, &envelope::encrypt(key, message)?, method), |_| {}).await
    }

    async fn store_session(&self, session: Session, wallet: Wallet, pairing_topic: &str, approval: Outgoing) -> Result<(), GemServiceError> {
        let connection = WalletConnection {
            session: self.connection_session(&session)?,
            wallet,
        };
        self.service.add_connection(connection).await?;
        self.store_and_send(approval, |state| {
            state.remove_pairing(pairing_topic);
            state.sessions.push(session);
        })
        .await
    }

    async fn store_and_send(&self, outgoing: Outgoing, change: impl FnOnce(&mut ClientState)) -> Result<(), GemServiceError> {
        self.update(|state| {
            change(state);
            state.outbox.push(outgoing.clone());
        })?;
        self.send_frame(outgoing.frame).await;
        Ok(())
    }

    async fn send_frame(&self, frame: String) {
        let _resent_once_connected = self.transport.send(frame).await;
    }

    async fn sync_connections(&self) -> Result<(), GemServiceError> {
        let sessions = self.read(|state| state.sessions.clone())?;
        let sessions = sessions.iter().map(|session| self.connection_session(session)).collect::<Result<Vec<_>, _>>()?;
        self.service.update_sessions(sessions).await
    }

    async fn verification(&self, message: &RelayMessage, decrypted: &[u8]) -> Verification {
        let attestation = match message.attestation.as_deref().filter(|jwt| !jwt.is_empty()) {
            Some(jwt) => self.attested(jwt, &verify::attestation_id(&message.message)).await,
            None => self.query_verify_server::<Attestation>(verify::attestation_url(decrypted)).await,
        };
        attestation.map_or_else(Verification::unknown, Attestation::verification)
    }

    fn request_verification(&self, message: &RelayMessage, session: &Session) -> Verification {
        message
            .attestation
            .as_deref()
            .filter(|jwt| !jwt.is_empty())
            .and_then(|jwt| self.stored_attestation(jwt, &verify::attestation_id(&message.message), now()))
            .map_or_else(
                || Verification {
                    origin: session.origin.clone(),
                    status: session.validation.clone(),
                },
                Attestation::verification,
            )
    }

    fn stored_attestation(&self, jwt: &str, attestation_id: &str, now: u64) -> Option<Attestation> {
        let key = self.read(|state| state.verify_key.clone()).ok().flatten().filter(|key| key.expires_at > now)?;
        verify::attested(jwt, &key, attestation_id, now)
    }

    async fn attested(&self, jwt: &str, attestation_id: &str) -> Option<Attestation> {
        let now = now();
        if let Some(attestation) = self.stored_attestation(jwt, attestation_id, now) {
            return Some(attestation);
        }
        let key: VerifyPublicKey = self.query_verify_server(verify::PUBLIC_KEY_URL.to_string()).await?;
        let attestation = verify::attested(jwt, &key, attestation_id, now);
        self.update_if_changed(|state| {
            let changed = state.verify_key.as_ref() != Some(&key);
            state.verify_key = Some(key);
            ((), changed)
        })
        .ok()?;
        attestation
    }

    async fn query_verify_server<T: DeserializeOwned>(&self, url: String) -> Option<T> {
        let target = AlienTarget {
            url,
            method: AlienHttpMethod::Get,
            headers: None,
            body: None,
        };
        let response = match select(Box::pin(self.provider.request(target)), Box::pin(sleep(VERIFY_TIMEOUT))).await {
            Either::Left((Ok(response), _)) => response.to_rpc_response(),
            Either::Left((Err(_), _)) | Either::Right(_) => return None,
        };
        match response.status {
            Some(HTTP_OK) => serde_json::from_slice(&response.data).ok(),
            _ => None,
        }
    }

    fn connection_session(&self, session: &Session) -> Result<WalletConnectionSession, GemServiceError> {
        self.service.session(
            session.topic.clone(),
            namespaces::accounts(&session.namespaces),
            i64::try_from(session.expiry).unwrap_or(i64::MAX),
            application_metadata(&session.peer),
        )
    }

    fn authentication_payload(&self, pending: &PendingAuthentication, wallet: Wallet) -> Result<(Account, Issuer, AuthPayload), GemServiceError> {
        let params = &pending.params;
        let app_url = &params.requester.metadata.url;
        if validate_domain(&params.auth_payload.domain, app_url).is_err() || self.service.is_origin_rejected(app_url.clone(), pending.verification.origin.clone(), pending.verification.status.clone()) {
            return Err(invalid_origin());
        }
        let accounts = self.service.authentication_accounts(params.auth_payload.chains.clone(), wallet);
        let selected = accounts.first().ok_or_else(|| GemServiceError::Unsupported {
            msg: "Requested chains are not supported".to_string(),
        })?;
        let supported_chains: Vec<String> = accounts.iter().map(|account| account.chain_id.clone()).collect();
        let payload = authenticate::narrowed_payload(&params.auth_payload, &supported_chains, &self.service.authentication_methods())?;
        Ok((selected.account.clone(), Issuer::new(&selected.chain_id, &selected.account.address), payload))
    }

    fn pairing_key(&self, topic: &str) -> Result<Key, GemServiceError> {
        let key = self.read(|state| state.pairings.iter().find(|pairing| pairing.topic == topic).map(|pairing| pairing.key.clone()))?;
        Ok(crypto::parse_key(&key.ok_or_else(|| not_found("pairing"))?)?)
    }

    fn metadata(&self) -> Metadata {
        let config = get_wallet_connect_config();
        Metadata {
            name: config.app_name,
            description: config.app_description,
            url: config.app_url,
            icons: config.app_icons,
            redirect: Some(Redirect {
                native: Some(rules::redirect(self.platform).to_string()),
                universal: None,
            }),
        }
    }

    fn read<R>(&self, read: impl FnOnce(&ClientState) -> R) -> Result<R, GemServiceError> {
        let mut state = lock(&self.state);
        Ok(read(self.loaded(&mut state)?))
    }

    fn update_in_memory<R>(&self, update: impl FnOnce(&mut ClientState) -> R) -> Result<R, GemServiceError> {
        let mut state = lock(&self.state);
        Ok(update(self.loaded(&mut state)?))
    }

    fn update<R>(&self, update: impl FnOnce(&mut ClientState) -> R) -> Result<R, GemServiceError> {
        self.update_if_changed(|state| (update(state), true))
    }

    fn update_if_changed<R>(&self, update: impl FnOnce(&mut ClientState) -> (R, bool)) -> Result<R, GemServiceError> {
        let mut state = lock(&self.state);
        let loaded = self.loaded(&mut state)?;
        let mut next = loaded.clone();
        let (result, changed) = update(&mut next);
        if changed {
            let json = serde_json::to_string(&next).map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
            self.store.set(STATE_KEY.to_string(), json)?;
        }
        *loaded = next;
        Ok(result)
    }

    fn loaded<'a>(&self, state: &'a mut Option<ClientState>) -> Result<&'a mut ClientState, GemServiceError> {
        if state.is_none() {
            *state = Some(match self.store.get(STATE_KEY.to_string())? {
                Some(json) => serde_json::from_str(&json).map_err(|error| GemServiceError::Store { msg: error.to_string() })?,
                None => ClientState::default(),
            });
        }
        Ok(state.get_or_insert_with(ClientState::default))
    }
}

impl From<ProtocolError> for GemServiceError {
    fn from(error: ProtocolError) -> Self {
        match error {
            ProtocolError::ExpiredUri => Self::Expired,
            ProtocolError::Random => Self::Core { msg: error.to_string() },
            ProtocolError::InvalidUri(_) | ProtocolError::InvalidKey | ProtocolError::InvalidEnvelope | ProtocolError::Decryption | ProtocolError::InvalidMessage(_) => Self::InvalidInput { msg: error.to_string() },
        }
    }
}

fn outgoing_publish(topic: &str, message: &str, method: Publish) -> Outgoing {
    let id = message_id();
    Outgoing {
        id,
        frame: rpc::publish(id, topic, message, method),
        expiry: now() + method.ttl(),
    }
}

fn authentication_answer(pending: &PendingAuthentication, secret: &Key, message: &[u8], method: Publish) -> Result<Outgoing, GemServiceError> {
    let requester_key = crypto::parse_key(&pending.params.requester.public_key)?;
    let key = crypto::shared_key(secret, &requester_key)?;
    let envelope = envelope::encrypt_with_sender_key(&key, &crypto::public_key(secret), message)?;
    Ok(outgoing_publish(&crypto::topic(requester_key.as_slice()), &envelope, method))
}

fn session_request(request: &PendingRequest) -> GemWalletConnectSessionRequest {
    GemWalletConnectSessionRequest {
        topic: request.topic.clone(),
        request_id: request.id.to_string(),
        method: request.method.clone(),
        params: request.params.clone(),
        chain_id: Some(request.chain_id.clone()),
        origin: request.origin.clone(),
        validation: request.validation.clone(),
        expiry: request.expiry,
    }
}

fn application_metadata(metadata: &Metadata) -> ApplicationMetadata {
    rules::application_metadata(metadata.name.clone(), metadata.description.clone(), metadata.url.clone(), metadata.icons.clone())
}

fn response_value(value: WalletConnectResponseType) -> Result<Box<RawValue>, GemServiceError> {
    let json = match value {
        WalletConnectResponseType::String { value } => serde_json::to_string(&value).map_err(|error| GemServiceError::Core { msg: error.to_string() })?,
        WalletConnectResponseType::Object { json } => json,
    };
    RawValue::from_string(json).map_err(|error| GemServiceError::Core { msg: error.to_string() })
}

fn reason(error: GemWalletConnectRpcError) -> Reason {
    Reason { code: error.code, message: error.message }
}

fn rejection(request_id: u64, rejection: &GemWalletConnectRejection) -> Vec<u8> {
    payload::failure(
        request_id,
        &Reason {
            code: rejection.code,
            message: rejection.message.clone(),
        },
    )
}

fn decode<T: DeserializeOwned>(params: &RawValue) -> Result<T, GemServiceError> {
    serde_json::from_str(params.get()).map_err(|error| ProtocolError::from(error).into())
}

fn parse_request_id(request_id: &str) -> Result<u64, GemServiceError> {
    request_id.parse().map_err(|_| GemServiceError::InvalidInput {
        msg: format!("invalid WalletConnect request id {request_id}"),
    })
}

fn invalid_origin() -> GemServiceError {
    GemServiceError::InvalidInput {
        msg: "WalletConnect origin does not match the app".to_string(),
    }
}

fn not_found(item: &str) -> GemServiceError {
    GemServiceError::NotFound {
        msg: format!("WalletConnect {item} not found"),
    }
}

async fn wait_until(expiry: u64) {
    while now() < expiry {
        sleep(EXPIRY_CHECK_INTERVAL).await;
    }
}

fn message_id() -> u64 {
    let random = random::bytes::<2>().map(|bytes| u64::from(u16::from_le_bytes(bytes) % 1000)).unwrap_or_default();
    now_millis() * 1000 + random
}

fn now() -> u64 {
    u64::try_from(Utc::now().timestamp()).unwrap_or_default()
}

fn now_millis() -> u64 {
    u64::try_from(Utc::now().timestamp_millis()).unwrap_or_default()
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::testkit::{TEST_AUTHENTICATION, TEST_PROPOSAL, TestDapp};
    use super::*;
    use crate::testkit::TestAlienProvider;
    use futures::FutureExt;
    use futures::executor::block_on;
    use primitives::{Chain, WalletConnectionVerificationStatus};
    use serde_json::Value;
    use wallet_connect_protocol::testkit::{TEST_PEER_PUBLIC_KEY, TEST_SYM_KEY, TEST_TOPIC, test_key, test_verify_key};

    fn json(value: &str) -> Value {
        serde_json::from_str(value).unwrap()
    }

    #[test]
    fn test_a_dapp_connects_and_its_request_is_answered() {
        block_on(async {
            let dapp = TestDapp::new();
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Polygon]);
            let (client, session_key) = GemWalletConnectClient::mock_connected(&dapp, wallet.clone()).await;
            let session_topic = TestDapp::topic(&session_key);
            let sent = dapp.transport.sent();

            assert_eq!(sent[0], json(&format!(r#"{{"id":{},"jsonrpc":"2.0","method":"irn_subscribe","params":{{"topic":"{TEST_TOPIC}"}}}}"#, sent[0]["id"])));
            assert_eq!(sent[1], json(r#"{"id":1,"jsonrpc":"2.0","result":true}"#), "every relay message is acknowledged");
            assert_eq!(dapp.listener.events.lock().unwrap()[0], GemWalletConnectEvent::Proposal { proposal: GemWalletConnectProposal::mock() });

            let approval = &sent[2];
            assert_eq!(approval["method"], "wc_approveSession");
            assert_eq!(approval["params"]["pairingTopic"], TEST_TOPIC);
            assert_eq!(approval["params"]["sessionTopic"], session_topic.as_str(), "the session topic is the hash of the key both sides derive");
            assert_eq!(approval["params"]["correlationId"], 100);
            let settle: Value = serde_json::from_slice(&envelope::decrypt(&session_key, approval["params"]["sessionSettlementRequest"].as_str().unwrap()).unwrap()).unwrap();
            assert_eq!(settle["method"], METHOD_SESSION_SETTLE);
            assert_eq!(
                settle["params"]["namespaces"],
                json(
                    r#"{"eip155":{"chains":["eip155:1","eip155:137"],"accounts":["eip155:1:address","eip155:137:address"],"methods":["eth_sendTransaction","personal_sign","eth_signTypedData_v4"],"events":["chainChanged","accountsChanged"]}}"#
                )
            );
            assert_eq!(settle["params"]["controller"]["metadata"]["redirect"], json(r#"{"native":"gem://wc"}"#));
            assert_eq!(
                client.service.store.get_connection(session_topic.clone()).await.unwrap().unwrap().wallet.id,
                wallet.id,
                "the connection is listed under the wallet that approved it"
            );
            assert_eq!(client.read(ClientState::topics).unwrap(), vec![session_topic.clone()], "the pairing is done once the session exists");

            let request = TestDapp::request(200, METHOD_SESSION_REQUEST, r#"{"request":{"method":"personal_sign","params":["0x48656c6c6f","0xaddress"]},"chainId":"eip155:1"}"#);
            client.receive(TestDapp::frame(&session_topic, &session_key, &request)).await.unwrap();
            assert_eq!(
                dapp.listener.events.lock().unwrap()[1],
                GemWalletConnectEvent::Request {
                    request: GemWalletConnectSessionRequest {
                        topic: session_topic.clone(),
                        request_id: "200".to_string(),
                        method: "personal_sign".to_string(),
                        params: r#"["0x48656c6c6f","0xaddress"]"#.to_string(),
                        chain_id: Some("eip155:1".to_string()),
                        origin: Some("https://example.com".to_string()),
                        validation: WalletConnectionVerificationStatus::Verified,
                        expiry: None,
                    }
                }
            );

            assert_eq!(client.answer_request(session_topic.clone(), "200".to_string()).await.unwrap(), None);
            assert_eq!(dapp.transport.published(&session_topic, &session_key), vec![json(r#"{"id":200,"jsonrpc":"2.0","result":"0xsignature"}"#)]);
            assert_eq!(client.read(|state| state.requests.len()).unwrap(), 0, "an answered request is no longer pending");

            client
                .receive(TestDapp::frame(&session_topic, &session_key, &TestDapp::request(300, METHOD_SESSION_DELETE, r#"{"code":6000,"message":"User disconnected."}"#)))
                .await
                .unwrap();
            assert_eq!(dapp.listener.events.lock().unwrap()[2], GemWalletConnectEvent::SessionDeleted { topic: session_topic.clone() });
            assert_eq!(dapp.transport.published(&session_topic, &session_key)[1], json(r#"{"id":300,"jsonrpc":"2.0","result":true}"#));
            assert!(client.service.store.get_connection(session_topic).await.unwrap().is_none());
            assert!(client.needs_connection().unwrap(), "answers the relay has not confirmed keep the socket wanted");
            client.receive(r#"{"id":1,"jsonrpc":"2.0","error":{"code":-32000,"message":"Rate limited"}}"#.to_string()).await.unwrap();
            assert!(client.needs_connection().unwrap(), "a frame the relay refused waits for the next connection");
            for frame in dapp.transport.sent().iter().filter(|frame| frame["method"] == "irn_publish" || frame["method"] == "wc_approveSession") {
                client.receive(TestDapp::reply(frame["id"].as_u64().unwrap())).await.unwrap();
            }
            assert!(!client.needs_connection().unwrap());
        });
    }

    #[test]
    fn test_a_session_answers_pings_and_refuses_what_it_did_not_approve() {
        block_on(async {
            let dapp = TestDapp::new();
            let (client, session_key) = GemWalletConnectClient::mock_connected(&dapp, Wallet::mock_with_chains(&[Chain::Ethereum])).await;
            let topic = TestDapp::topic(&session_key);
            let receive = |payload: String| client.receive(TestDapp::frame(&topic, &session_key, &payload));

            receive(TestDapp::request(1, METHOD_SESSION_PING, "{}")).await.unwrap();
            receive(TestDapp::request(2, METHOD_SESSION_REQUEST, r#"{"request":{"method":"eth_signTypedData_v4","params":[]},"chainId":"eip155:1"}"#))
                .await
                .unwrap();
            receive(TestDapp::request(3, METHOD_SESSION_REQUEST, r#"{"request":{"method":"personal_sign","params":[]},"chainId":"eip155:56"}"#))
                .await
                .unwrap();
            receive(TestDapp::request(4, METHOD_SESSION_REQUEST, r#"{"request":{"method":"personal_sign","params":[],"expiryTimestamp":1},"chainId":"eip155:1"}"#))
                .await
                .unwrap();

            assert_eq!(
                dapp.transport.published(&topic, &session_key),
                vec![
                    json(r#"{"id":1,"jsonrpc":"2.0","result":true}"#),
                    json(r#"{"id":2,"jsonrpc":"2.0","error":{"code":3001,"message":"Unauthorized JSON-RPC method requested: eth_signTypedData_v4"}}"#),
                    json(r#"{"id":3,"jsonrpc":"2.0","error":{"code":3005,"message":"Unauthorized target chain id requested"}}"#),
                    json(r#"{"id":4,"jsonrpc":"2.0","error":{"code":8000,"message":"Session request expired"}}"#),
                ]
            );
            assert_eq!(dapp.listener.events.lock().unwrap().len(), 1, "only the proposal reached the app");
        });
    }

    #[test]
    fn test_a_rejected_or_unsupported_proposal_is_answered_on_its_pairing() {
        block_on(async {
            for (wallet, reason, expected) in [
                (
                    Wallet::mock(),
                    Some(GemWalletConnectRejectionReason::UserRejected),
                    r#"{"id":100,"jsonrpc":"2.0","error":{"code":5000,"message":"User rejected."}}"#,
                ),
                (Wallet::mock_with_chains(&[Chain::Solana]), None, r#"{"id":100,"jsonrpc":"2.0","error":{"code":5100,"message":"Unsupported chains"}}"#),
            ] {
                let dapp = TestDapp::new();
                let pairing_key = test_key(TEST_SYM_KEY);
                let client = GemWalletConnectClient::mock_with_service(&dapp, Platform::IOS, GemWalletConnectService::mock(Ok(String::new()), wallet.clone()).await);
                client.pair(TestDapp::pairing_uri("wc_sessionPropose")).await.unwrap();
                client.receive(TestDapp::frame(TEST_TOPIC, &pairing_key, &TestDapp::request(100, METHOD_SESSION_PROPOSE, TEST_PROPOSAL))).await.unwrap();

                match reason {
                    Some(reason) => client.reject_session(TEST_PEER_PUBLIC_KEY.to_string(), reason).await.unwrap(),
                    None => assert_eq!(
                        client.approve_session(TEST_PEER_PUBLIC_KEY.to_string(), wallet).await,
                        Err(GemServiceError::Unsupported { msg: "Unsupported chains".to_string() })
                    ),
                }

                assert_eq!(dapp.transport.published(TEST_TOPIC, &pairing_key), vec![json(expected)]);
                assert!(client.read(|state| state.pairings.is_empty()).unwrap(), "a proposal answered once leaves no pairing behind");
                assert_eq!(
                    client.reject_session(TEST_PEER_PUBLIC_KEY.to_string(), GemWalletConnectRejectionReason::UserRejected).await,
                    Ok(()),
                    "a proposal answered once is gone"
                );
            }
        });
    }

    fn return_events(dapp: &TestDapp) -> Vec<GemWalletConnectEvent> {
        dapp.listener.events.lock().unwrap().iter().filter(|event| matches!(event, GemWalletConnectEvent::ReturnToDapp { .. })).cloned().collect()
    }

    fn return_to_example(redirect: Option<&str>) -> GemWalletConnectEvent {
        GemWalletConnectEvent::ReturnToDapp {
            name: "Example".to_string(),
            redirect: redirect.map(str::to_string),
        }
    }

    #[test]
    fn test_a_proposal_opened_from_another_app_returns_to_it_once_answered() {
        block_on(async {
            let with_redirect = TEST_PROPOSAL.replace(
                r#""url": "https://example.com","#,
                r#""url": "https://example.com", "redirect": {"native": "example://wc", "universal": "https://example.com/app"},"#,
            );
            for (proposal, link, approves, expected) in [
                (TEST_PROPOSAL.to_string(), true, true, vec![return_to_example(None)]),
                (with_redirect.clone(), true, true, vec![return_to_example(Some("example://wc"))]),
                (with_redirect.clone(), true, false, vec![return_to_example(Some("example://wc"))]),
                (with_redirect, false, true, vec![]),
            ] {
                let dapp = TestDapp::new();
                let wallet = Wallet::mock_with_chains(&[Chain::Ethereum]);
                let client = GemWalletConnectClient::mock_with_service(&dapp, Platform::IOS, GemWalletConnectService::mock(Ok(String::new()), wallet.clone()).await);
                let uri = TestDapp::pairing_uri("wc_sessionPropose");
                if link {
                    client.link_opened(WalletConnectLink::Connect { uri: uri.clone() });
                }
                client.pair(uri).await.unwrap();
                client.receive(TestDapp::frame(TEST_TOPIC, &test_key(TEST_SYM_KEY), &TestDapp::request(100, METHOD_SESSION_PROPOSE, &proposal))).await.unwrap();
                assert!(return_events(&dapp).is_empty(), "nothing returns before the user answers");

                if approves {
                    client.approve_session(TEST_PEER_PUBLIC_KEY.to_string(), wallet).await.unwrap();
                } else {
                    client.reject_session(TEST_PEER_PUBLIC_KEY.to_string(), GemWalletConnectRejectionReason::UserRejected).await.unwrap();
                }

                assert_eq!(return_events(&dapp), expected, "a code scanned from another device stays in Gem");
            }
        });
    }

    #[test]
    fn test_a_request_opened_from_another_app_returns_to_it_after_the_last_waiting_request() {
        block_on(async {
            let dapp = TestDapp::new();
            let (client, session_key) = GemWalletConnectClient::mock_connected(&dapp, Wallet::mock_with_chains(&[Chain::Ethereum])).await;
            let topic = TestDapp::topic(&session_key);
            let receive = |id: u64| {
                client.receive(TestDapp::frame(
                    &topic,
                    &session_key,
                    &TestDapp::request(id, METHOD_SESSION_REQUEST, r#"{"request":{"method":"personal_sign","params":["0x48656c6c6f","0xaddress"]},"chainId":"eip155:1"}"#),
                ))
            };

            receive(200).await.unwrap();
            client.answer_request(topic.clone(), "200".to_string()).await.unwrap();
            assert!(return_events(&dapp).is_empty(), "a request from a dapp on another device stays in Gem");

            client.link_opened(WalletConnectLink::Session { topic: topic.clone() });
            receive(201).await.unwrap();
            receive(202).await.unwrap();
            client.answer_request(topic.clone(), "201".to_string()).await.unwrap();
            assert!(return_events(&dapp).is_empty(), "another request of the same dapp is still waiting");
            client.reject_request(topic.clone(), "202".to_string()).await.unwrap();
            assert_eq!(return_events(&dapp), vec![return_to_example(None)], "a rejection returns too, so the dapp shows it");

            receive(203).await.unwrap();
            client.answer_request(topic.clone(), "203".to_string()).await.unwrap();
            assert_eq!(return_events(&dapp).len(), 1, "one link returns once");

            client.link_opened(WalletConnectLink::Session { topic: topic.clone() });
            client.disconnect(topic.clone()).await.unwrap();
            client.link_opened(WalletConnectLink::Request);
            assert!(lock(&client.linked_topics).is_empty(), "a disconnected session forgets its link");
        });
    }

    #[test]
    fn test_restore_shows_unanswered_requests_one_at_a_time_and_drops_connections_without_a_session() {
        block_on(async {
            let dapp = TestDapp::new();
            let (client, session_key) = GemWalletConnectClient::mock_connected(&dapp, Wallet::mock_with_chains(&[Chain::Ethereum])).await;
            let topic = TestDapp::topic(&session_key);
            for id in [201, 200] {
                let request = TestDapp::request(id, METHOD_SESSION_REQUEST, r#"{"request":{"method":"personal_sign","params":["0x48656c6c6f","0xaddress"]},"chainId":"eip155:1"}"#);
                client.receive(TestDapp::frame(&topic, &session_key, &request)).await.unwrap();
            }

            let restored_dapp = dapp.relaunched();
            let service = GemWalletConnectService::mock(Ok("0xsignature".to_string()), Wallet::mock_with_chains(&[Chain::Ethereum])).await;
            let restored = GemWalletConnectClient::mock_with_service(&restored_dapp, Platform::IOS, service);
            restored.restore().await.unwrap();
            restored.restore().await.unwrap();

            let request_ids = |dapp: &TestDapp| -> Vec<String> {
                dapp.listener
                    .events
                    .lock()
                    .unwrap()
                    .iter()
                    .filter_map(|event| match event {
                        GemWalletConnectEvent::Request { request } => Some(request.request_id.clone()),
                        _ => None,
                    })
                    .collect()
            };
            assert_eq!(request_ids(&restored_dapp), vec!["200"], "the oldest request left open when the app closed shows first, alone and once");
            assert!(restored.service.store.get_connection("topic".to_string()).await.unwrap().is_none(), "a stored connection without a live session is removed");

            restored.answer_request(topic.clone(), "200".to_string()).await.unwrap();
            assert_eq!(request_ids(&restored_dapp), vec!["200", "201"], "the next one shows once the first is answered");

            restored.connected().await.unwrap();
            let sent = restored_dapp.transport.sent();
            assert_eq!(sent.iter().find(|frame| frame["method"] == "irn_batchSubscribe").unwrap()["params"]["topics"], json(&format!(r#"["{topic}"]"#)));
            assert_eq!(
                sent.iter().filter(|frame| frame["method"] == "wc_approveSession").count(),
                1,
                "a frame the relay never confirmed is sent again on the next connection"
            );
        });
    }

    #[test]
    fn test_an_app_closed_while_its_answer_is_sent_neither_asks_again_nor_loses_the_answer() {
        block_on(async {
            let dapp = TestDapp::new();
            let (client, session_key) = GemWalletConnectClient::mock_connected(&dapp, Wallet::mock_with_chains(&[Chain::Ethereum])).await;
            let topic = TestDapp::topic(&session_key);
            let request = TestDapp::request(200, METHOD_SESSION_REQUEST, r#"{"request":{"method":"personal_sign","params":["0x48656c6c6f","0xaddress"]},"chainId":"eip155:1"}"#);
            client.receive(TestDapp::frame(&topic, &session_key, &request)).await.unwrap();
            dapp.transport.stalled.store(true, std::sync::atomic::Ordering::SeqCst);

            assert!(client.answer_request(topic.clone(), "200".to_string()).now_or_never().is_none(), "the app closes while the signed answer is on its way");

            let relaunched_dapp = dapp.relaunched();
            let relaunched = GemWalletConnectClient::mock(&relaunched_dapp, Platform::IOS).await;
            relaunched.restore().await.unwrap();
            relaunched.connected().await.unwrap();

            assert!(relaunched_dapp.listener.events.lock().unwrap().is_empty(), "a signed request is never shown again after the app closes");
            assert_eq!(relaunched_dapp.transport.published(&topic, &session_key), vec![json(r#"{"id":200,"jsonrpc":"2.0","result":"0xsignature"}"#)]);
        });
    }

    #[test]
    fn test_an_expired_proposal_is_not_shown() {
        block_on(async {
            let dapp = TestDapp::new();
            let client = GemWalletConnectClient::mock(&dapp, Platform::IOS).await;
            client.pair(TestDapp::pairing_uri("wc_sessionPropose")).await.unwrap();
            let proposal = TEST_PROPOSAL.replace("4102444800", "1");

            client.receive(TestDapp::frame(TEST_TOPIC, &test_key(TEST_SYM_KEY), &TestDapp::request(100, METHOD_SESSION_PROPOSE, &proposal))).await.unwrap();

            assert!(dapp.listener.events.lock().unwrap().is_empty());
        });
    }

    #[test]
    fn test_a_request_delivered_again_after_its_answer_is_ignored() {
        block_on(async {
            let dapp = TestDapp::new();
            let (client, session_key) = GemWalletConnectClient::mock_connected(&dapp, Wallet::mock_with_chains(&[Chain::Ethereum])).await;
            let topic = TestDapp::topic(&session_key);
            let request = TestDapp::frame(
                &topic,
                &session_key,
                &TestDapp::request(200, METHOD_SESSION_REQUEST, r#"{"request":{"method":"personal_sign","params":["0x48656c6c6f","0xaddress"]},"chainId":"eip155:1"}"#),
            );

            client.receive(request.clone()).await.unwrap();
            client.answer_request(topic.clone(), "200".to_string()).await.unwrap();
            client.receive(request).await.unwrap();

            assert_eq!(dapp.listener.events.lock().unwrap().len(), 2, "the proposal and one request");
            assert!(client.read(|state| state.requests.is_empty()).unwrap());
        });
    }

    #[test]
    fn test_a_request_answered_twice_reaches_the_dapp_once() {
        block_on(async {
            let dapp = TestDapp::new();
            let (client, session_key) = GemWalletConnectClient::mock_connected(&dapp, Wallet::mock_with_chains(&[Chain::Ethereum])).await;
            let topic = TestDapp::topic(&session_key);
            let request = TestDapp::request(200, METHOD_SESSION_REQUEST, r#"{"request":{"method":"personal_sign","params":["0x48656c6c6f","0xaddress"]},"chainId":"eip155:1"}"#);
            client.receive(TestDapp::frame(&topic, &session_key, &request)).await.unwrap();
            let (pending, session) = client.read(|state| state.request(&topic, 200)).unwrap().unwrap();

            client.reject_request(topic.clone(), "200".to_string()).await.unwrap();
            let signature = GemWalletConnectResponse::Response {
                value: WalletConnectResponseType::String { value: "0xsignature".to_string() },
            };
            client.respond(&pending, &session, signature).await.unwrap();

            let rejected = rules::user_rejected_error();
            assert_eq!(
                dapp.transport.published(&topic, &session_key),
                vec![serde_json::json!({"id": 200, "jsonrpc": "2.0", "error": {"code": rejected.code, "message": rejected.message}})],
                "a signature finished after the user rejected the request is not sent"
            );
        });
    }

    #[test]
    fn test_a_proposal_is_answered_once() {
        block_on(async {
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum]);
            let pairing_key = test_key(TEST_SYM_KEY);
            let proposal = TestDapp::frame(TEST_TOPIC, &pairing_key, &TestDapp::request(100, METHOD_SESSION_PROPOSE, TEST_PROPOSAL));

            let dapp = TestDapp::new();
            let client = GemWalletConnectClient::mock_with_service(&dapp, Platform::IOS, GemWalletConnectService::mock(Ok(String::new()), wallet.clone()).await);
            client.pair(TestDapp::pairing_uri("wc_sessionPropose")).await.unwrap();
            client.receive(proposal.clone()).await.unwrap();
            client.receive(TestDapp::frame(TEST_TOPIC, &pairing_key, &TestDapp::request(101, METHOD_SESSION_PROPOSE, TEST_PROPOSAL))).await.unwrap();
            assert_eq!(dapp.listener.events.lock().unwrap().len(), 1, "a pairing shows one proposal at a time");

            client.reject_session(TEST_PEER_PUBLIC_KEY.to_string(), GemWalletConnectRejectionReason::UserRejected).await.unwrap();
            assert!(matches!(client.approve_session(TEST_PEER_PUBLIC_KEY.to_string(), wallet.clone()).await, Err(GemServiceError::NotFound { .. })));
            assert_eq!(dapp.transport.published(TEST_TOPIC, &pairing_key), vec![json(r#"{"id":100,"jsonrpc":"2.0","error":{"code":5000,"message":"User rejected."}}"#)]);
            assert!(!dapp.transport.sent().iter().any(|frame| frame["method"] == "wc_approveSession"), "a rejected proposal is never approved");

            let dapp = TestDapp::new();
            let client = GemWalletConnectClient::mock_with_service(&dapp, Platform::IOS, GemWalletConnectService::mock(Ok(String::new()), wallet.clone()).await);
            client.pair(TestDapp::pairing_uri("wc_sessionPropose")).await.unwrap();
            client.receive(proposal).await.unwrap();
            client.receive(TestDapp::frame(TEST_TOPIC, &pairing_key, &TestDapp::request(101, METHOD_PAIRING_DELETE, "{}"))).await.unwrap();
            assert!(
                matches!(client.approve_session(TEST_PEER_PUBLIC_KEY.to_string(), wallet).await, Err(GemServiceError::NotFound { .. })),
                "a proposal whose pairing the dapp deleted can no longer be approved"
            );
        });
    }

    #[test]
    fn test_a_connection_approved_while_its_approval_is_sent_is_listed() {
        block_on(async {
            let dapp = TestDapp::new();
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum]);
            let client = GemWalletConnectClient::mock_with_service(&dapp, Platform::IOS, GemWalletConnectService::mock(Ok(String::new()), wallet.clone()).await);
            client.pair(TestDapp::pairing_uri("wc_sessionPropose")).await.unwrap();
            client
                .receive(TestDapp::frame(TEST_TOPIC, &test_key(TEST_SYM_KEY), &TestDapp::request(100, METHOD_SESSION_PROPOSE, TEST_PROPOSAL)))
                .await
                .unwrap();
            dapp.transport.stalled.store(true, std::sync::atomic::Ordering::SeqCst);

            assert!(client.approve_session(TEST_PEER_PUBLIC_KEY.to_string(), wallet).now_or_never().is_none());

            let topic = client.read(ClientState::topics).unwrap().remove(0);
            assert!(
                client.service.store.get_connection(topic).await.unwrap().is_some(),
                "a dapp told it is connected is listed, so its requests can be signed and it can be disconnected"
            );
        });
    }

    #[test]
    fn test_a_request_attested_as_a_scam_is_refused_on_a_verified_session() {
        block_on(async {
            let dapp = TestDapp::new();
            let (client, session_key) = GemWalletConnectClient::mock_connected(&dapp, Wallet::mock_with_chains(&[Chain::Ethereum])).await;
            let topic = TestDapp::topic(&session_key);
            client.update(|state| state.verify_key = Some(test_verify_key(4_102_444_800))).unwrap();
            let request = TestDapp::request(200, METHOD_SESSION_REQUEST, r#"{"request":{"method":"personal_sign","params":["0x48656c6c6f","0xaddress"]},"chainId":"eip155:1"}"#);

            client.receive(TestDapp::attested_frame(&topic, &session_key, &request, true)).await.unwrap();

            assert!(matches!(
                &dapp.listener.events.lock().unwrap()[1],
                GemWalletConnectEvent::Request { request } if request.validation == WalletConnectionVerificationStatus::Malicious
            ));
            assert_eq!(client.answer_request(topic.clone(), "200".to_string()).await.unwrap(), Some(GemWalletConnectFailure::MaliciousOrigin));
            assert!(dapp.transport.published(&topic, &session_key)[0]["error"].is_object(), "nothing is signed for a site flagged after it connected");
        });
    }

    #[test]
    fn test_a_deleted_session_closes_its_open_requests() {
        block_on(async {
            let dapp = TestDapp::new();
            let (client, session_key) = GemWalletConnectClient::mock_connected(&dapp, Wallet::mock_with_chains(&[Chain::Ethereum])).await;
            let topic = TestDapp::topic(&session_key);
            let request = TestDapp::request(200, METHOD_SESSION_REQUEST, r#"{"request":{"method":"personal_sign","params":["0x48656c6c6f","0xaddress"]},"chainId":"eip155:1"}"#);
            client.receive(TestDapp::frame(&topic, &session_key, &request)).await.unwrap();

            client
                .receive(TestDapp::frame(&topic, &session_key, &TestDapp::request(300, METHOD_SESSION_DELETE, r#"{"code":6000,"message":"User disconnected."}"#)))
                .await
                .unwrap();

            assert_eq!(
                dapp.listener.events.lock().unwrap()[2..].to_vec(),
                vec![
                    GemWalletConnectEvent::RequestExpired {
                        topic: topic.clone(),
                        request_id: "200".to_string()
                    },
                    GemWalletConnectEvent::SessionDeleted { topic },
                ],
                "a review whose dapp is gone closes before the user signs something no one will hear about"
            );
        });
    }

    #[test]
    fn test_a_request_that_could_not_be_stored_is_delivered_again() {
        block_on(async {
            let dapp = TestDapp::new();
            let (client, session_key) = GemWalletConnectClient::mock_connected(&dapp, Wallet::mock_with_chains(&[Chain::Ethereum])).await;
            let topic = TestDapp::topic(&session_key);
            let frame = TestDapp::frame(
                &topic,
                &session_key,
                &TestDapp::request(200, METHOD_SESSION_REQUEST, r#"{"request":{"method":"personal_sign","params":["0x48656c6c6f","0xaddress"]},"chainId":"eip155:1"}"#),
            );
            let acknowledgements = || dapp.transport.sent().iter().filter(|frame| frame["id"] == 1 && frame["result"] == true).count();
            let acknowledged = acknowledgements();
            dapp.store.failing.store(true, std::sync::atomic::Ordering::SeqCst);

            assert!(matches!(client.receive(frame.clone()).await, Err(GemServiceError::Platform { .. })));
            assert_eq!(acknowledgements(), acknowledged, "the relay keeps a message the wallet could not store");

            dapp.store.failing.store(false, std::sync::atomic::Ordering::SeqCst);
            client.receive(frame).await.unwrap();
            assert_eq!(dapp.listener.events.lock().unwrap().len(), 2, "the proposal and the request delivered again");
            assert_eq!(acknowledgements(), acknowledged + 1);
        });
    }

    #[test]
    fn test_core_refuses_a_proposal_or_sign_in_for_another_site() {
        block_on(async {
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "0x1111111111111111111111111111111111111111")]);
            let pairing_key = test_key(TEST_SYM_KEY);
            let dapp = TestDapp::new();
            let provider: Arc<dyn AlienProvider> = Arc::new(TestAlienProvider::with_json_by_path(200, &[("verify.walletconnect.org", r#"{"origin":"https://evil.example","isScam":false}"#)]));
            let service = GemWalletConnectService::mock(Ok(String::new()), wallet.clone()).await;
            let client = GemWalletConnectClient::new(Arc::new(service), dapp.store.clone(), provider, dapp.transport.clone(), dapp.listener.clone(), Platform::IOS);
            client.pair(TestDapp::pairing_uri("wc_sessionPropose")).await.unwrap();
            client.receive(TestDapp::frame(TEST_TOPIC, &pairing_key, &TestDapp::request(100, METHOD_SESSION_PROPOSE, TEST_PROPOSAL))).await.unwrap();

            assert!(matches!(client.approve_session(TEST_PEER_PUBLIC_KEY.to_string(), wallet.clone()).await, Err(GemServiceError::InvalidInput { .. })));
            assert_eq!(dapp.transport.published(TEST_TOPIC, &pairing_key), vec![json(r#"{"id":100,"jsonrpc":"2.0","error":{"code":5000,"message":"User rejected."}}"#)]);
            assert!(!dapp.transport.sent().iter().any(|frame| frame["method"] == "wc_approveSession"), "a page that is not the app it names is never connected");

            let dapp = TestDapp::new();
            let client = GemWalletConnectClient::mock_with_service(&dapp, Platform::Android, GemWalletConnectService::mock(Ok(String::new()), wallet.clone()).await);
            client.pair(TestDapp::pairing_uri("wc_sessionPropose,wc_sessionAuthenticate")).await.unwrap();
            let authentication = TEST_AUTHENTICATION.replace(r#""domain": "example.com""#, r#""domain": "app.uniswap.org""#);
            assert_ne!(authentication, TEST_AUTHENTICATION);
            client.receive(TestDapp::frame(TEST_TOPIC, &pairing_key, &TestDapp::request(101, METHOD_SESSION_AUTHENTICATE, &authentication))).await.unwrap();

            assert!(matches!(client.authentication_message("101".to_string(), wallet.clone()), Err(GemServiceError::InvalidInput { .. })));
            assert!(
                matches!(client.approve_authentication("101".to_string(), wallet, "0xsignature".to_string()).await, Err(GemServiceError::InvalidInput { .. })),
                "a sign-in for another site is never signed or sent"
            );
            assert!(dapp.transport.published_with_sender_key(&TestDapp::requester_topic()).is_empty());
        });
    }

    #[test]
    fn test_unreadable_state_is_an_error() {
        block_on(async {
            let dapp = TestDapp::new();
            dapp.store.set(STATE_KEY.to_string(), "{".to_string()).unwrap();
            let client = GemWalletConnectClient::mock(&dapp, Platform::IOS).await;

            assert!(matches!(client.needs_connection(), Err(GemServiceError::Store { .. })), "sessions are never dropped because their record could not be read");
        });
    }

    #[test]
    fn test_pair() {
        block_on(async {
            let dapp = TestDapp::new();
            let client = GemWalletConnectClient::mock(&dapp, Platform::IOS).await;
            let subscriptions = || dapp.transport.sent().iter().filter(|frame| frame["method"] == "irn_subscribe").count();

            assert_eq!(client.pair(format!("{}&expiryTimestamp=1", TestDapp::pairing_uri("wc_sessionPropose"))).await, Err(GemServiceError::Expired));
            assert_eq!(
                client.pair("wc:abc@2?relay-protocol=irn".to_string()).await,
                Err(GemServiceError::InvalidInput {
                    msg: "invalid WalletConnect URI: topic".to_string()
                })
            );

            client.pair(TestDapp::pairing_uri("wc_sessionPropose")).await.unwrap();
            client.pair(TestDapp::pairing_uri("wc_sessionPropose")).await.unwrap();
            assert_eq!(subscriptions(), 2, "a code opened again before its proposal arrived subscribes again");

            client
                .receive(TestDapp::frame(TEST_TOPIC, &test_key(TEST_SYM_KEY), &TestDapp::request(100, METHOD_SESSION_PROPOSE, TEST_PROPOSAL)))
                .await
                .unwrap();
            client.pair(TestDapp::pairing_uri("wc_sessionPropose")).await.unwrap();
            assert_eq!(subscriptions(), 2, "the same code opened while its proposal is on screen leaves the proposal there");
            assert_eq!(dapp.listener.events.lock().unwrap().len(), 1);
            assert!(client.read(|state| state.pairings[0].expiry > now()).unwrap(), "a code without an expiry still expires");
        });
    }

    #[test]
    fn test_one_click_sign_in_opens_a_session_for_the_signing_wallet() {
        block_on(async {
            let dapp = TestDapp::new();
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "0x1111111111111111111111111111111111111111")]);
            let client = GemWalletConnectClient::mock_with_service(&dapp, Platform::Android, GemWalletConnectService::mock(Ok(String::new()), wallet.clone()).await);
            let pairing_key = test_key(TEST_SYM_KEY);
            client.pair(TestDapp::pairing_uri("wc_sessionPropose,wc_sessionAuthenticate")).await.unwrap();

            client.receive(TestDapp::frame(TEST_TOPIC, &pairing_key, &TestDapp::request(100, METHOD_SESSION_PROPOSE, TEST_PROPOSAL))).await.unwrap();
            client
                .receive(TestDapp::frame(TEST_TOPIC, &pairing_key, &TestDapp::request(101, METHOD_SESSION_AUTHENTICATE, TEST_AUTHENTICATION)))
                .await
                .unwrap();

            assert_eq!(
                dapp.listener.events.lock().unwrap().clone(),
                vec![GemWalletConnectEvent::Authentication {
                    authentication: GemWalletConnectAuthentication {
                        id: "101".to_string(),
                        metadata: rules::application_metadata("Example".to_string(), "Example dapp".to_string(), "https://example.com".to_string(), vec!["https://example.com/icon.png".to_string()]),
                        chain_ids: vec!["eip155:1".to_string(), "eip155:137".to_string()],
                        domain: "example.com".to_string(),
                        origin: Some("https://example.com".to_string()),
                        validation: WalletConnectionVerificationStatus::Verified,
                    }
                }],
                "the fallback proposal of a one-click sign-in is not shown"
            );

            let message = client.authentication_message("101".to_string(), wallet.clone()).unwrap();
            assert_eq!(message.account, wallet.accounts[0]);
            assert_eq!(message.message.lines().next(), Some("example.com wants you to sign in with your Ethereum account:"));

            client.approve_authentication("101".to_string(), wallet.clone(), "0xsignature".to_string()).await.unwrap();

            let response = dapp.transport.published_with_sender_key(&TestDapp::requester_topic()).remove(0);
            assert_eq!(response["id"], 101);
            let cacao = &response["result"]["cacaos"][0];
            assert_eq!(cacao["p"]["iss"], "did:pkh:eip155:1:0x1111111111111111111111111111111111111111");
            assert_eq!(cacao["s"], json(r#"{"t":"eip191","s":"0xsignature"}"#));
            let session_topic = TestDapp::topic(&TestDapp::session_key(response["result"]["responder"]["publicKey"].as_str().unwrap()));
            assert_eq!(client.read(ClientState::topics).unwrap(), vec![session_topic.clone()]);
            assert_eq!(client.service.store.get_connection(session_topic.clone()).await.unwrap().unwrap().wallet.id, wallet.id);
            assert!(
                dapp.transport.sent().iter().any(|frame| frame["method"] == "irn_subscribe" && frame["params"]["topic"] == session_topic.as_str()),
                "the wallet listens on the new session"
            );
        });
    }

    #[test]
    fn test_one_click_sign_in_closed_while_its_approval_is_sent_keeps_its_session() {
        block_on(async {
            let dapp = TestDapp::new();
            let wallet = Wallet::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "0x1111111111111111111111111111111111111111")]);
            let client = GemWalletConnectClient::mock_with_service(&dapp, Platform::Android, GemWalletConnectService::mock(Ok(String::new()), wallet.clone()).await);
            client.pair(TestDapp::pairing_uri("wc_sessionPropose,wc_sessionAuthenticate")).await.unwrap();
            client
                .receive(TestDapp::frame(TEST_TOPIC, &test_key(TEST_SYM_KEY), &TestDapp::request(101, METHOD_SESSION_AUTHENTICATE, TEST_AUTHENTICATION)))
                .await
                .unwrap();
            dapp.transport.stalled.store(true, std::sync::atomic::Ordering::SeqCst);

            assert!(client.approve_authentication("101".to_string(), wallet, "0xsignature".to_string()).now_or_never().is_none());

            let relaunched_dapp = dapp.relaunched();
            let relaunched = GemWalletConnectClient::mock(&relaunched_dapp, Platform::Android).await;
            relaunched.connected().await.unwrap();
            let response = relaunched_dapp.transport.published_with_sender_key(&TestDapp::requester_topic()).remove(0);
            let session_topic = TestDapp::topic(&TestDapp::session_key(response["result"]["responder"]["publicKey"].as_str().unwrap()));
            assert_eq!(relaunched.read(ClientState::topics).unwrap(), vec![session_topic.clone()], "the session the dapp is told about exists on the wallet");
            assert_eq!(
                relaunched_dapp.transport.sent().iter().find(|frame| frame["method"] == "irn_batchSubscribe").unwrap()["params"]["topics"],
                json(&format!(r#"["{session_topic}"]"#))
            );
        });
    }

    #[test]
    fn test_ios_ignores_one_click_sign_in() {
        block_on(async {
            let dapp = TestDapp::new();
            let client = GemWalletConnectClient::mock(&dapp, Platform::IOS).await;
            client.pair(TestDapp::pairing_uri("wc_sessionPropose,wc_sessionAuthenticate")).await.unwrap();

            client
                .receive(TestDapp::frame(TEST_TOPIC, &test_key(TEST_SYM_KEY), &TestDapp::request(101, METHOD_SESSION_AUTHENTICATE, TEST_AUTHENTICATION)))
                .await
                .unwrap();
            client
                .receive(TestDapp::frame(TEST_TOPIC, &test_key(TEST_SYM_KEY), &TestDapp::request(100, METHOD_SESSION_PROPOSE, TEST_PROPOSAL)))
                .await
                .unwrap();

            assert_eq!(
                dapp.listener.events.lock().unwrap().clone(),
                vec![GemWalletConnectEvent::Proposal { proposal: GemWalletConnectProposal::mock() }],
                "iOS connects through the fallback proposal"
            );
        });
    }
}
