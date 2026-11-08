use std::collections::BTreeMap;

use primitives::WalletConnectionVerificationStatus;
use serde::{Deserialize, Serialize};
use wallet_connect_protocol::sign::{Metadata, SessionNamespace};
use wallet_connect_protocol::verify::VerifyPublicKey;

const ANSWER_MEMORY_SECONDS: u64 = 86_400;

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(default)]
pub struct ClientState {
    pub pairings: Vec<Pairing>,
    pub sessions: Vec<Session>,
    pub requests: Vec<PendingRequest>,
    pub answered: Vec<AnsweredRequest>,
    pub outbox: Vec<Outgoing>,
    pub verify_key: Option<VerifyPublicKey>,
    #[serde(skip)]
    pub shown: Option<(String, u64)>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct Pairing {
    pub topic: String,
    pub key: String,
    pub expiry: u64,
    pub authenticates: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub struct Session {
    pub topic: String,
    pub key: String,
    pub peer: Metadata,
    pub namespaces: BTreeMap<String, SessionNamespace>,
    pub expiry: u64,
    pub origin: Option<String>,
    pub validation: WalletConnectionVerificationStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingRequest {
    pub topic: String,
    pub id: u64,
    pub method: String,
    pub params: String,
    pub chain_id: String,
    pub expiry: Option<u64>,
    pub origin: Option<String>,
    pub validation: WalletConnectionVerificationStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnsweredRequest {
    pub topic: String,
    pub id: u64,
    pub expiry: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Outgoing {
    pub id: u64,
    pub frame: String,
    pub expiry: u64,
}

pub enum Topic {
    Pairing(Pairing),
    Session(Session),
}

impl ClientState {
    pub fn topic(&self, topic: &str) -> Option<Topic> {
        self.sessions
            .iter()
            .find(|session| session.topic == topic)
            .cloned()
            .map(Topic::Session)
            .or_else(|| self.pairings.iter().find(|pairing| pairing.topic == topic).cloned().map(Topic::Pairing))
    }

    pub fn topics(&self) -> Vec<String> {
        self.sessions.iter().map(|session| session.topic.clone()).chain(self.pairings.iter().map(|pairing| pairing.topic.clone())).collect()
    }

    pub fn session(&self, topic: &str) -> Option<&Session> {
        self.sessions.iter().find(|session| session.topic == topic)
    }

    pub fn request(&self, topic: &str, id: u64) -> Option<(PendingRequest, Session)> {
        let request = self.requests.iter().find(|request| request.topic == topic && request.id == id)?;
        Some((request.clone(), self.session(topic)?.clone()))
    }

    pub fn add_request(&mut self, request: PendingRequest) -> bool {
        let is_known = self.is_waiting(&request.topic, request.id) || self.answered.iter().any(|answered| answered.topic == request.topic && answered.id == request.id);
        if !is_known {
            self.shown = Some((request.topic.clone(), request.id));
            self.requests.push(request);
        }
        !is_known
    }

    pub fn show_next_request(&mut self) -> Option<PendingRequest> {
        if self.shown.as_ref().is_some_and(|(topic, id)| self.is_waiting(topic, *id)) {
            return None;
        }
        let request = self.requests.iter().filter(|request| self.session(&request.topic).is_some()).min_by_key(|request| request.id)?.clone();
        self.shown = Some((request.topic.clone(), request.id));
        Some(request)
    }

    fn is_waiting(&self, topic: &str, id: u64) -> bool {
        self.requests.iter().any(|request| request.topic == topic && request.id == id)
    }

    pub fn answer(&mut self, topic: &str, id: u64, answer: Outgoing, now: u64) -> bool {
        let Some(index) = self.requests.iter().position(|pending| pending.topic == topic && pending.id == id) else {
            return false;
        };
        let request = self.requests.remove(index);
        self.answered.push(AnsweredRequest {
            topic: request.topic,
            id: request.id,
            expiry: request.expiry.unwrap_or_default().max(now + ANSWER_MEMORY_SECONDS),
        });
        self.outbox.push(answer);
        true
    }

    pub fn remove_pairing(&mut self, topic: &str) {
        self.pairings.retain(|pairing| pairing.topic != topic);
    }

    pub fn remove_session(&mut self, topic: &str) -> Option<(Session, Vec<PendingRequest>)> {
        let index = self.sessions.iter().position(|session| session.topic == topic)?;
        let (dropped, kept) = std::mem::take(&mut self.requests).into_iter().partition(|request| request.topic == topic);
        self.requests = kept;
        Some((self.sessions.remove(index), dropped))
    }

    pub fn prune(&mut self, now: u64) -> bool {
        let count = self.pairings.len() + self.sessions.len() + self.requests.len() + self.answered.len() + self.outbox.len();
        self.pairings.retain(|pairing| pairing.expiry > now);
        self.sessions.retain(|session| session.expiry > now);
        let topics: Vec<String> = self.sessions.iter().map(|session| session.topic.clone()).collect();
        self.requests.retain(|request| topics.contains(&request.topic) && request.expiry.is_none_or(|expiry| expiry > now));
        self.answered.retain(|answered| answered.expiry > now);
        self.outbox.retain(|outgoing| outgoing.expiry > now);
        count != self.pairings.len() + self.sessions.len() + self.requests.len() + self.answered.len() + self.outbox.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prune() {
        let mut state = ClientState {
            pairings: vec![Pairing::mock("live", 200), Pairing::mock("expired", 100)],
            sessions: vec![Session::mock("live", 200), Session::mock("expired", 100)],
            requests: vec![
                PendingRequest::mock("live", 1, Some(200)),
                PendingRequest::mock("live", 2, Some(100)),
                PendingRequest::mock("live", 3, None),
                PendingRequest::mock("expired", 4, None),
            ],
            answered: vec![AnsweredRequest::mock(5, 200), AnsweredRequest::mock(6, 100)],
            outbox: vec![Outgoing::mock(7, 200), Outgoing::mock(8, 100)],
            verify_key: None,
            shown: None,
        };

        assert!(state.prune(100));
        assert!(!state.prune(100), "nothing left to drop is not a change worth storing");

        assert_eq!(state.topics(), vec!["live", "live"]);
        assert_eq!(
            state.requests,
            vec![PendingRequest::mock("live", 1, Some(200)), PendingRequest::mock("live", 3, None)],
            "requests outlive neither their expiry nor their session"
        );
        assert_eq!(state.answered, vec![AnsweredRequest::mock(5, 200)]);
        assert_eq!(state.outbox, vec![Outgoing::mock(7, 200)], "an answer the relay never confirmed is dropped once its time to live is over");
    }

    #[test]
    fn test_answer() {
        let mut state = ClientState {
            requests: vec![PendingRequest::mock("topic", 1, Some(500_000)), PendingRequest::mock("topic", 2, None)],
            ..ClientState::default()
        };

        assert!(state.answer("topic", 1, Outgoing::mock(10, 200), 100));
        assert!(state.answer("topic", 2, Outgoing::mock(11, 200), 100));
        assert!(!state.answer("topic", 1, Outgoing::mock(12, 200), 100), "a request is answered once");

        assert!(state.requests.is_empty());
        assert_eq!(
            state.answered,
            vec![AnsweredRequest::mock(1, 500_000), AnsweredRequest::mock(2, 86_500)],
            "an answer is remembered while the request could still be delivered again"
        );
        assert_eq!(state.outbox, vec![Outgoing::mock(10, 200), Outgoing::mock(11, 200)]);
        assert!(!state.add_request(PendingRequest::mock("topic", 1, None)), "a request delivered again after its answer is not asked again");
        assert!(state.add_request(PendingRequest::mock("topic", 3, None)));
        assert!(!state.add_request(PendingRequest::mock("topic", 3, None)));
    }

    #[test]
    fn test_show_next_request() {
        let mut state = ClientState {
            sessions: vec![Session::mock("topic", 200)],
            requests: vec![PendingRequest::mock("topic", 2, None), PendingRequest::mock("topic", 1, None)],
            ..ClientState::default()
        };

        assert_eq!(state.show_next_request().map(|request| request.id), Some(1), "the oldest unanswered request shows first");
        assert_eq!(state.show_next_request(), None, "one request at a time");

        state.answer("topic", 1, Outgoing::mock(10, 200), 100);
        assert_eq!(state.show_next_request().map(|request| request.id), Some(2));
        assert_eq!(state.show_next_request(), None);

        let mut stored: ClientState = serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert_eq!(stored.show_next_request().map(|request| request.id), Some(2), "a request shown before the app closed shows again after it opens");
    }

    #[test]
    fn test_a_request_replaced_on_screen_shows_again_once_the_newer_one_is_answered() {
        let mut state = ClientState {
            sessions: vec![Session::mock("topic", 200)],
            ..ClientState::default()
        };

        assert!(state.add_request(PendingRequest::mock("topic", 1, None)));
        assert!(state.add_request(PendingRequest::mock("topic", 2, None)));
        assert_eq!(state.show_next_request(), None, "the newer request is on screen");

        state.answer("topic", 2, Outgoing::mock(10, 200), 100);
        assert_eq!(state.show_next_request().map(|request| request.id), Some(1), "a request the app never answered is not left waiting");
    }

    #[test]
    fn test_state_written_before_a_field_was_added_still_reads() {
        let stored: ClientState = serde_json::from_str(r#"{"sessions":[]}"#).unwrap();

        assert_eq!(stored, ClientState::default());
    }

    #[test]
    fn test_remove_session() {
        let mut state = ClientState {
            sessions: vec![Session::mock("one", 200), Session::mock("two", 200)],
            requests: vec![PendingRequest::mock("one", 1, None), PendingRequest::mock("two", 2, None)],
            ..ClientState::default()
        };

        assert_eq!(state.remove_session("one"), Some((Session::mock("one", 200), vec![PendingRequest::mock("one", 1, None)])));
        assert_eq!(state.remove_session("one"), None);
        assert_eq!(state.requests, vec![PendingRequest::mock("two", 2, None)], "a deleted session takes its unanswered requests with it");
    }
}
