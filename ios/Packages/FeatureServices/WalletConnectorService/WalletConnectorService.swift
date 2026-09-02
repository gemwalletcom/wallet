// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemChainService
import Foundation
import enum Gemstone.GemWalletConnectFailure
import struct Gemstone.GemWalletConnectSessionRequest
import enum Gemstone.GemWalletConnectError
import protocol Gemstone.GemWalletConnectServiceProtocol
import GemstonePrimitives
import protocol Gemstone.GemWalletSessionServiceProtocol
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

            let outcome = await service.processRequest(request: GemWalletConnectSessionRequest(
                topic: request.topic,
                requestId: request.id.string,
                method: request.method,
                params: (try? JSONEncoder().encode(request.params).encodeString()) ?? "",
                chainId: request.chainId.absoluteString,
                origin: verifyContext?.origin,
                validation: verifyContext?.validation.map() ?? .unknown,
            ))
            do {
                try await WalletKit.instance.respond(topic: request.topic, requestId: request.id, response: outcome.response.map())
            } catch {
                debugLog("Error responding to request: \(error)")
            }
            if let failure = outcome.failure {
                await walletConnectorInteractor.sessionReject(error: failure.error)
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

    private func processSession(proposal: Session.Proposal, verifyContext: VerifyContext) async throws {
        let messageId = proposal.messageId

        guard service.shouldProcessMessage(messageId: messageId) else {
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
            chains: approval.chains.compactMap { $0.blockchain(chainService: chainService) },
            methods: approval.methods,
            events: approval.events,
            accounts: approval.accounts.compactMap { $0.blockchain(chainService: chainService) },
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

private extension GemWalletConnectFailure {
    var error: any Error {
        switch self {
        case .maliciousOrigin: GemWalletConnectError.InvalidOrigin
        case let .failed(message): AnyError(message)
        }
    }
}
