// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemWalletConnectRequest
import protocol Gemstone.GemWalletConnectServiceProtocol
import GemstonePrimitives
import protocol GemstoneServices.WalletSessionManageable
import Primitives
@preconcurrency import ReownWalletKit
@preconcurrency import WalletConnectPairing

public final class WalletConnectorService {
    private let interactor = WCConnectionsInteractor()
    private let walletSessionService: any WalletSessionManageable
    private let walletConnectorInteractor: any WalletConnectorInteractable
    private let service: any GemWalletConnectServiceProtocol
    private let messageTracker = MessageTracker()
    private let setupState = SetupState()

    public init(
        walletSessionService: any WalletSessionManageable,
        interactor: any WalletConnectorInteractable,
        service: any GemWalletConnectServiceProtocol,
    ) {
        self.walletSessionService = walletSessionService
        walletConnectorInteractor = interactor
        self.service = service
    }
}

// MARK: - WalletConnectorService

extension WalletConnectorService: WalletConnectorServiceable {
    public func configure() throws {
        Networking.configure(
            groupIdentifier: Constants.WalletConnect.groupIdentifier,
            projectId: Constants.WalletConnect.projectId,
            socketFactory: DefaultSocketFactory(),
        )

        try WalletKit.configure(
            metadata: AppMetadata(
                name: Constants.App.name,
                description: "Gem Web3 Wallet",
                url: Constants.App.website,
                icons: ["https://gemwallet.com/images/gem-logo-256x256.png"],
                redirect: AppMetadata.Redirect(
                    native: "gem://",
                    universal: .none,
                ),
            ),
            crypto: DefaultCryptoProvider(),
        )
    }

    public func setup() async {
        guard await setupState.start() else {
            return
        }
        Events.instance.setTelemetryEnabled(false)
        await withTaskGroup(of: Void.self) { group in
            group.addTask {
                await self.handleSessions()
            }

            group.addTask {
                await self.handleSessionProposals()
            }

            group.addTask {
                await self.handleSessionRequests()
            }

            group.addTask {
                await self.handleSessionDeletes()
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
    private func handleSessions() async {
        for await sessions in interactor.sessionsStream {
            await updateSessions(sessions)
        }
    }

    private func handleSessionProposals() async {
        for await (proposal, verifyContext) in interactor.sessionProposalStream {
            debugLog("Session proposal received: \(proposal)")
            debugLog("Verify context: \(String(describing: verifyContext))")

            guard let verifyContext else {
                await handleRejectSession(proposal: proposal, error: WalletConnectorServiceError.invalidOrigin)
                continue
            }

            do {
                try await processSession(proposal: proposal, verifyContext: verifyContext)
            } catch {
                debugLog("Error accepting proposal: \(error)")

                await handleRejectSession(proposal: proposal, error: error)
            }
        }
    }

    private func handleRejectSession(proposal: Session.Proposal, error: Error) async {
        try? await WalletKit.instance.rejectSession(
            proposalId: proposal.id,
            reason: RejectionReason(from: error),
        )
        try? await service.deleteSession(sessionId: proposal.pairingTopic)
        await walletConnectorInteractor.sessionReject(error: error)
    }

    private func handleSessionRequests() async {
        for await (request, verifyContext) in interactor.sessionRequestStream {
            debugLog("Session request received: \(request.method)")
            debugLog("Verify context: \(String(describing: verifyContext))")

            let session = WalletKit.instance.getSessions().first { $0.topic == request.topic }

            guard let verifyContext, let session else {
                try? await rejectRequest(request)
                continue
            }

            do {
                let status = service.validateOrigin(metadataUrl: session.peer.url, origin: verifyContext.origin, validation: verifyContext.validation.map()).map()

                debugLog("Verification status for request: \(status)")

                switch status {
                case .verified, .unknown: break
                case .invalid, .malicious:
                    // show toast with an error
                    debugLog("Warning: Request status error (\(status)")
                    try await rejectRequest(request)
                    continue
                }

                try await handleRequest(request: request, session: session)
            } catch {
                debugLog("Error handling request: \(error)")
            }
        }
    }

    private func handleSessionDeletes() async {
        for await deletion in interactor.sessionDeleteStream {
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
        try service.metadata(name: metadata.name, description: metadata.description, url: metadata.url, icons: metadata.icons)
    }

    private func handleRequest(request: WalletConnectSign.Request, session: Session) async throws {
        let messageId = request.messageId

        guard await messageTracker.shouldProcess(messageId) else {
            debugLog("Ignoring duplicate request with ID: \(messageId)")
            try await rejectRequest(request)
            return
        }

        debugLog("handleMethod received: \(request.method), params: \(request.params)")

        do {
            let params = try JSONEncoder().encode(request.params).encodeString()
            let response = try await service.handleRequest(
                request: GemWalletConnectRequest(
                    topic: request.topic,
                    method: request.method,
                    params: params,
                    chainId: request.chainId.absoluteString,
                    domain: session.peer.url,
                ),
            )
            debugLog("handle method result: \(request.method) \(response)")
            try await WalletKit.instance.respond(topic: request.topic, requestId: request.id, response: response.map())
        } catch let requestError {
            debugLog("handle method error: \(requestError)")
            do {
                try await rejectRequest(request)
            } catch {
                debugLog("Error rejecting request: \(error)")
            }
            await walletConnectorInteractor.sessionReject(error: requestError)
        }
    }

    private func rejectRequest(_ request: WalletConnectSign.Request) async throws {
        let rejection = service.userRejectedError()
        try await WalletKit.instance.respond(
            topic: request.topic,
            requestId: request.id,
            response: .error(JSONRPCError(code: Int(rejection.code), message: rejection.message)),
        )
    }

    private func processSession(proposal: Session.Proposal, verifyContext: VerifyContext) async throws {
        let messageId = proposal.messageId

        guard await messageTracker.shouldProcess(messageId) else {
            debugLog("Ignoring duplicate proposal with ID: \(messageId)")
            return
        }

        let (payload, status) = try service.prepareSessionProposal(
            requiredChainIds: proposal.requiredNamespaces.chainIds,
            optionalChainIds: proposal.optionalNamespaces?.chainIds ?? [],
            metadata: metadata(proposal.proposer),
            origin: verifyContext.origin,
            validation: verifyContext.validation.map(),
        )
        debugLog("Verification status: \(status)")
        let payloadTopic = WCPairingProposal(
            pairingId: proposal.pairingTopic,
            proposal: payload,
            verificationStatus: status.map(),
        )
        let approvedWalletId = try await walletConnectorInteractor.sessionApproval(payload: payloadTopic)
        let selectedWallet = try walletSessionService.getWallet(walletId: approvedWalletId)

        let session = try await acceptProposal(proposal: proposal, wallet: selectedWallet)
        try await service.addConnection(WalletConnection(session: connectionSession(session), wallet: selectedWallet))
    }

    private func acceptProposal(proposal: Session.Proposal, wallet: Primitives.Wallet) async throws -> Session {
        let approval = try service.sessionApproval(wallet: wallet)
        let sessionNamespaces = try AutoNamespaces.build(
            sessionProposal: proposal,
            chains: approval.chains.compactMap(\.blockchain),
            methods: approval.methods,
            events: approval.events,
            accounts: approval.accounts.compactMap(\.blockchain),
        )
        let caip2Chains = sessionNamespaces.values.flatMap { $0.chains ?? [] }.map(\.absoluteString)
        let sessionProperties = service.configSessionProperties(
            properties: proposal.sessionProperties ?? [:],
            caip2Chains: caip2Chains,
            accounts: approval.accounts.map { $0.mapToGem() },
        )
        return try await WalletKit.instance.approve(
            proposalId: proposal.id,
            namespaces: sessionNamespaces,
            sessionProperties: sessionProperties,
        )
    }
}
