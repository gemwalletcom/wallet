// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import enum Gemstone.SignDigestType
import struct Gemstone.SignMessage
import class Gemstone.WalletConnect
import enum Gemstone.WalletConnectAction
import enum Gemstone.WalletConnectChainOperation
import enum Gemstone.WalletConnectResponseType
import class Gemstone.WalletConnectSimulationClient
import enum Gemstone.SignableTransaction
import enum Gemstone.SignableTransactionType
import GemstonePrimitives
import NativeProviderService
import Primitives
import SigningRequestService
@preconcurrency import ReownWalletKit
@preconcurrency import WalletConnectPairing

public final class WalletConnectorService {
    private let interactor = WCConnectionsInteractor()
    private let signer: WalletConnectorSignable
    private let messageTracker = MessageTracker()
    private let walletConnect = WalletConnect()
    private let simulator: SigningSimulator

    public init(
        signer: WalletConnectorSignable,
        nodeProvider: any NodeURLFetchable,
        requestInterceptor: any RequestInterceptable = EmptyRequestInterceptor(),
    ) {
        self.signer = signer
        simulator = SigningSimulator(nodeProvider: nodeProvider, requestInterceptor: requestInterceptor)
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
        let uri = try WalletConnectURI(uriString: uri)
        try await Pair.instance.pair(uri: uri)
    }

    public func disconnect(sessionId: String) async throws {
        try await WalletKit.instance.disconnect(topic: sessionId)
    }

    public func updateSessions() {
        updateSessions(interactor.sessions)
    }
}

// MARK: - Private

extension WalletConnectorService {
    private func simulateSignMessage(chain: Gemstone.Chain, signType: SignDigestType, data: String, sessionDomain: String) async throws -> Primitives.SimulationResult {
        try await simulator.simulateSignMessage(chain: chain, signType: signType, data: data, sessionDomain: sessionDomain)
    }

    private func simulateSendTransaction(
        chain: Gemstone.Chain,
        transactionType: SignableTransactionType,
        data: String,
    ) async throws -> Primitives.SimulationResult {
        try await simulator.simulateSendTransaction(chain: chain, transactionType: transactionType, data: data)
    }

    private func handleSessions() async {
        for await sessions in interactor.sessionsStream {
            updateSessions(sessions)
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
        try? await signer.sessionReject(id: proposal.pairingTopic, error: error)
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
                let status = walletConnect.validateOrigin(metadataUrl: session.peer.metadata.url, origin: verifyContext.origin, validation: verifyContext.validation.map()).map()

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

    private func updateSessions(_ sessions: [Session]) {
        debugLog("Received sessions: \(sessions)")
        do {
            try signer.updateSessions(sessions: sessions.map(\.asSession))
        } catch {
            debugLog("Error updating sessions: \(error)")
        }
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
            let action = try walletConnect.parseRequest(
                topic: request.topic,
                method: request.method,
                params: params,
                chainId: request.chainId.absoluteString,
                domain: session.peer.metadata.url,
            )

            debugLog("parse request result: \(action)")

            let response = try await handleAction(action: action, sessionId: request.topic, sessionDomain: session.peer.metadata.url)

            debugLog("handle method result: \(request.method) \(response)")
            try await WalletKit.instance.respond(topic: request.topic, requestId: request.id, response: response)
        } catch let requestError {
            debugLog("handle method error: \(requestError)")
            do {
                try await rejectRequest(request)
            } catch {
                debugLog("Error rejecting request: \(error)")
            }
            await signer.sessionReject(error: requestError)
        }
    }

    private func handleAction(action: WalletConnectAction, sessionId: String, sessionDomain: String) async throws -> RPCResult {
        switch action {
        case let .signMessage(chain, signType, data):
            let simulation = try await simulateSignMessage(chain: chain, signType: signType, data: data, sessionDomain: sessionDomain)
            let message = walletConnect.decodeSignMessage(chain: chain, signType: signType, data: data)
            let signature = try await signer.signMessage(
                sessionId: sessionId,
                chain: chain.map(),
                message: message,
                simulation: simulation,
            )
            let response = walletConnect.encodeSignMessage(chain: chain, signature: signature)
            return .response(response.map())
        case let .signTransaction(chain, type, data):
            let simulation = try await simulateSendTransaction(chain: chain, transactionType: type, data: data)
            let transaction = try walletConnect.decodeSendTransaction(transactionType: type, data: data)
            let transactionId = try await signer.signTransaction(sessionId: sessionId, chain: chain.map(), transaction: transaction.map(), simulation: simulation)
            let response = walletConnect.encodeSignTransaction(chain: chain, transactionId: transactionId)
            return .response(response.map())
        case let .signAllTransactions(chain, type, transactions):
            guard transactions.count <= 1, let data = transactions.first else {
                throw WalletConnectorServiceError.unresolvedMethod("signAllTransactions with multiple transactions is not yet supported")
            }
            let simulation = try await simulateSendTransaction(chain: chain, transactionType: type, data: data)
            let transaction = try walletConnect.decodeSendTransaction(transactionType: type, data: data)
            let signed = try await signer.signTransaction(sessionId: sessionId, chain: chain.map(), transaction: transaction.map(), simulation: simulation)
            let response = walletConnect.encodeSignAllTransactions(signedTransactions: [signed])
            return .response(response.map())
        case let .sendTransaction(chain, type, data):
            let simulation = try await simulateSendTransaction(chain: chain, transactionType: type, data: data)
            let transaction = try walletConnect.decodeSendTransaction(transactionType: type, data: data)
            let transactionId = try await signer.sendTransaction(
                sessionId: sessionId,
                chain: chain.map(),
                transaction: transaction.map(),
                simulation: simulation,
            )
            let response = walletConnect.encodeSendTransaction(chain: chain, transactionId: transactionId)
            return .response(response.map())
        case let .chainOperation(operation):
            return handleChainOperation(operation: operation)
        case let .getAccounts(chain):
            let accounts = try signer.getAccounts(sessionId: sessionId, chain: chain.map())
            let response = walletConnect.encodeGetAccounts(chain: chain, accounts: accounts.map { $0.mapToGem() })
            return .response(response.map())
        case .unsupported:
            return .error(.methodNotFound)
        }
    }

    private func handleChainOperation(operation: WalletConnectChainOperation) -> RPCResult {
        switch operation {
        case .addChain, .switchChain: .response(AnyCodable.null())
        case .getChainId: .error(.methodNotFound)
        }
    }

    private func rejectRequest(_ request: WalletConnectSign.Request) async throws {
        try await WalletKit.instance.respond(topic: request.topic, requestId: request.id, response: .error(JSONRPCError(code: 4001, message: "User rejected the request")))
    }

    private func processSession(proposal: Session.Proposal, verifyContext: VerifyContext) async throws {
        let messageId = proposal.messageId

        guard await messageTracker.shouldProcess(messageId) else {
            debugLog("Ignoring duplicate proposal with ID: \(messageId)")
            return
        }

        let wallets = try signer.getWallets(for: proposal)
        let currentWalletId = try signer.getCurrentWallet().id

        guard let preselectedWallet = wallets.first(where: { $0.id == currentWalletId }) ?? wallets.first else {
            throw WalletConnectorServiceError.walletsUnsupported
        }

        let metadata = proposal.proposer.metadata
        let status = walletConnect.validateOrigin(metadataUrl: metadata.url, origin: verifyContext.origin, validation: verifyContext.validation.map()).map()

        debugLog("Verification status: \(status)")

        switch status {
        case .verified, .unknown: break
        case .invalid, .malicious:
            throw WalletConnectorServiceError.invalidOrigin
        }

        let payload = WalletConnectSessionProposal(
            defaultWallet: preselectedWallet,
            wallets: wallets,
            metadata: metadata,
        )

        let payloadTopic = WCPairingProposal(
            pairingId: proposal.pairingTopic,
            proposal: payload,
            verificationStatus: status,
        )
        let approvedWalletId = try await signer.sessionApproval(payload: payloadTopic)
        let selectedWallet = try signer.getWallet(id: approvedWalletId)

        let session = try await acceptProposal(proposal: proposal, wallet: selectedWallet)
        try signer.addConnection(connection: WalletConnection(session: session.asSession, wallet: selectedWallet))
    }

    private func acceptProposal(proposal: Session.Proposal, wallet: Wallet) async throws -> Session {
        let chains = signer.getChains(wallet: wallet)
        let accounts = wallet.accounts.filter { chains.contains($0.chain) }
        let events = signer.getEvents()
        let methods = signer.getMethods()
        let supportedAccounts = accounts.compactMap(\.blockchain)
        let supportedChains = chains.compactMap(\.blockchain)

        let sessionNamespaces = try AutoNamespaces.build(
            sessionProposal: proposal,
            chains: supportedChains,
            methods: methods.map(\.rawValue),
            events: events.map(\.rawValue),
            accounts: supportedAccounts,
        )
        let caip2Chains = sessionNamespaces.values.flatMap { $0.chains ?? [] }.map(\.absoluteString)
        let sessionProperties = walletConnect.configSessionProperties(
            properties: proposal.sessionProperties ?? [:],
            caip2Chains: caip2Chains,
            accounts: accounts.map { $0.mapToGem() },
        )

        return try await WalletKit.instance.approve(
            proposalId: proposal.id,
            namespaces: sessionNamespaces,
            sessionProperties: sessionProperties,
        )
    }
}
