// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemChainService
import enum Gemstone.GemWalletConnectError
import enum Gemstone.GemWalletConnectFailure
import enum Gemstone.GemWalletConnectRejectionReason
import protocol Gemstone.GemWalletConnectServiceProtocol
import struct Gemstone.GemWalletConnectSessionRequest
import protocol Gemstone.GemWalletSessionServiceProtocol
import GemstonePrimitives
import Primitives
@preconcurrency import ReownWalletKit
@preconcurrency import WalletConnectPairing

public final class WalletConnectorService {
    private let interactor = WCConnectionsInteractor()
    private let walletSessionService: any GemWalletSessionServiceProtocol
    private let walletConnectorInteractor: any WalletConnectorInteractable
    private let service: any GemWalletConnectServiceProtocol
    private let chainService: GemChainService
    private let setupState = SetupState()

    public init(
        walletSessionService: any GemWalletSessionServiceProtocol,
        interactor: any WalletConnectorInteractable,
        service: any GemWalletConnectServiceProtocol,
        chainService: GemChainService,
    ) {
        self.walletSessionService = walletSessionService
        walletConnectorInteractor = interactor
        self.service = service
        self.chainService = chainService
    }
}

// MARK: - WalletConnectorService

extension WalletConnectorService: WalletConnectorServiceable {
    public func configure() throws {
        let config = WalletConnectConfig.config()
        Networking.configure(
            groupIdentifier: Constants.appGroupIdentifier,
            projectId: config.projectId,
            socketFactory: DefaultSocketFactory(),
        )

        try WalletKit.configure(
            metadata: AppMetadata(
                name: config.appName,
                description: config.appDescription,
                url: config.appUrl,
                icons: config.appIcons,
                redirect: AppMetadata.Redirect(
                    native: "gem://",
                    universal: .none,
                ),
            ),
            crypto: DefaultCryptoProvider(),
        )
    }

    public func setup() async {
        await setupState.start {
            Events.instance.setTelemetryEnabled(false)
            let sessionsStream = UncheckedSendable(value: self.interactor.sessionsStream)
            let sessionProposalStream = UncheckedSendable(value: self.interactor.sessionProposalStream)
            let sessionRequestStream = UncheckedSendable(value: self.interactor.sessionRequestStream)
            let sessionDeleteStream = UncheckedSendable(value: self.interactor.sessionDeleteStream)

            _ = Task {
                await self.observeSessions(sessionsStream.value)
            }
            _ = Task {
                await self.observeSessionProposals(sessionProposalStream.value)
            }
            _ = Task {
                await self.observeSessionRequests(sessionRequestStream.value)
            }
            _ = Task {
                await self.observeSessionDeletes(sessionDeleteStream.value)
            }
        }
    }

    public func pair(uri: String) async throws {
        await setup()
        let uri = try WalletConnectURI(uriString: uri)
        try await Pair.instance.pair(uri: uri)
    }

    public func disconnect(sessionId: String) async throws {
        try await service.deleteSession(sessionId: sessionId)
        try await WalletKit.instance.disconnect(topic: sessionId)
    }

    public func updateSessions() {
        Task {
            await updateSessions(interactor.sessions)
        }
    }

    public func hasSessions() async throws -> Bool {
        try await service.hasSessions()
    }
}

// MARK: - Private

extension WalletConnectorService {
    private func observeSessions(_ stream: AsyncStream<[Session]>) async {
        for await sessions in stream {
            await updateSessions(sessions)
        }
    }

    private func observeSessionProposals(_ stream: AsyncStream<(proposal: Session.Proposal, context: VerifyContext?)>) async {
        for await (proposal, verifyContext) in stream {
            debugLog("Session proposal received: \(proposal)")
            debugLog("Verify context: \(String(describing: verifyContext))")

            do {
                try await approveSession(proposal: proposal, verifyContext: verifyContext)
            } catch {
                debugLog("Error accepting proposal: \(error)")

                await rejectSession(proposal: proposal, error: error)
            }
        }
    }

    private func rejectSession(proposal: Session.Proposal, error: Error) async {
        let rejection = service.sessionRejection(reason: GemWalletConnectRejectionReason(from: error))
        do {
            try await WalletKit.instance.rejectSession(
                proposalId: proposal.id,
                reason: RejectionReason(rejection.reason),
            )
        } catch {
            debugLog("Error rejecting proposal: \(error)")
        }
        if rejection.deletesSession {
            do {
                try await service.deleteSession(sessionId: proposal.pairingTopic)
            } catch {
                debugLog("Error deleting rejected session: \(error)")
            }
        }
        await walletConnectorInteractor.sessionReject(error: error)
    }

    private func observeSessionRequests(_ stream: AsyncStream<(request: Request, context: VerifyContext?)>) async {
        for await (request, verifyContext) in stream {
            debugLog("Session request received: \(request.method) chain: \(request.chainId.absoluteString)")
            debugLog("Verify context: \(String(describing: verifyContext))")

            let params: String
            do {
                params = try JSONEncoder().encode(request.params).encodeString()
                debugLog("Session request params: \(params)")
            } catch {
                debugLog("Error encoding request params: \(error) raw: \(request.params.stringRepresentation)")
                await rejectRequest(request, error: error)
                continue
            }

            let outcome = await service.requestOutcome(request: GemWalletConnectSessionRequest(
                topic: request.topic,
                requestId: request.id.string,
                method: request.method,
                params: params,
                chainId: request.chainId.absoluteString,
                origin: verifyContext?.origin,
                validation: verifyContext?.validation.map() ?? .unknown,
                expiry: request.expiryTimestamp,
            ))
            if let response = outcome.response {
                do {
                    try await WalletKit.instance.respond(topic: request.topic, requestId: request.id, response: response.map())
                } catch {
                    debugLog("Error responding to request: \(error)")
                }
            }
            if let failure = outcome.failure {
                debugLog("Session request failed: \(String(describing: failure)) params: \(params)")
                await walletConnectorInteractor.sessionReject(error: failure.error)
            }
        }
    }

    private func rejectRequest(_ request: Request, error: Error) async {
        do {
            try await WalletKit.instance.respond(
                topic: request.topic,
                requestId: request.id,
                response: .error(JSONRPCError(code: Int(GemConstants.walletConnectUserRejectedErrorCode), message: GemConstants.walletConnectUserRejectedErrorMessage)),
            )
        } catch {
            debugLog("Error rejecting request: \(error)")
        }
        await walletConnectorInteractor.sessionReject(error: error)
    }

    private func observeSessionDeletes(_ stream: AsyncStream<(topic: String, code: Int, message: String)>) async {
        for await deletion in stream {
            debugLog("Session deleted by peer: topic: \(deletion.topic), reason: \(deletion.message) (code: \(deletion.code))")
        }
    }

    private func updateSessions(_ sessions: [Session]) async {
        debugLog("Received sessions: \(sessions)")
        do {
            try await service.updateSessions(sessions.map { try connectionSession($0) })
        } catch {
            debugLog("Error updating sessions: \(error)")
        }
    }

    private func connectionSession(_ session: Session) throws -> WalletConnectionSession {
        try service.session(
            topic: session.topic,
            accounts: session.namespaces.values.flatMap(\.accounts).map(\.absoluteString),
            expireAt: session.expiryDate,
            metadata: metadata(session.peer),
        )
    }

    private func metadata(_ metadata: AppMetadata) throws -> ApplicationMetadata {
        service.metadata(name: metadata.name, description: metadata.description, url: metadata.url, icons: metadata.icons)
    }

    private func approveSession(proposal: Session.Proposal, verifyContext: VerifyContext?) async throws {
        guard service.shouldProcessProposal(proposerPublicKey: proposal.id) else {
            debugLog("Ignoring duplicate proposal with ID: \(proposal.id)")
            return
        }

        let (payload, status) = try await service.prepareSessionProposal(
            requiredChainIds: proposal.requiredNamespaces.chainIds,
            optionalChainIds: proposal.optionalNamespaces?.chainIds ?? [],
            metadata: metadata(proposal.proposer),
            origin: verifyContext?.origin,
            validation: verifyContext?.validation.map() ?? .unknown,
        )
        debugLog("Verification status: \(status)")
        let payloadTopic = WCPairingProposal(
            pairingId: proposal.pairingTopic,
            proposal: payload,
            verificationStatus: status.toPrimitives(),
        )
        let approvedWalletId = try await walletConnectorInteractor.sessionApproval(payload: payloadTopic)
        let selectedWallet = try await walletSessionService.requireWallet(walletId: approvedWalletId)

        let session = try await acceptProposal(proposal: proposal, wallet: selectedWallet)
        try await service.addConnection(WalletConnection(session: connectionSession(session), wallet: selectedWallet))
    }

    private func acceptProposal(proposal: Session.Proposal, wallet: Primitives.Wallet) async throws -> Session {
        let approval = service.sessionApproval(wallet: wallet.toGem())
        let sessionNamespaces = try AutoNamespaces.build(
            sessionProposal: proposal,
            chains: approval.chains.compactMap { Primitives.Chain(core: $0).blockchain(chainService: chainService) },
            methods: approval.methods,
            events: approval.events,
            accounts: approval.accounts.compactMap { $0.toPrimitives().blockchain(chainService: chainService) },
        )
        let caip2Chains = sessionNamespaces.values.flatMap { $0.chains ?? [] }.map(\.absoluteString)
        let sessionProperties = service.configSessionProperties(
            properties: proposal.sessionProperties ?? [:],
            caip2Chains: caip2Chains,
            accounts: approval.accounts,
        )
        return try await WalletKit.instance.approve(
            proposalId: proposal.id,
            namespaces: sessionNamespaces,
            sessionProperties: sessionProperties,
        )
    }
}
