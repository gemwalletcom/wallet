// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.Config
import class Gemstone.MessageSigner
import struct Gemstone.SignMessage
import Preferences
import Primitives
import SigningRequestService
import Store
import WalletConnectorService
import WalletConnectSign
import WalletSessionService

public final class WalletConnectorSigner: WalletConnectorSignable {
    private let connectionsStore: ConnectionsStore
    private let walletConnectorInteractor: any WalletConnectorInteractable
    private let signingInteractor: any SigningRequestInteractable
    private let walletSessionService: any WalletSessionManageable

    public init(
        connectionsStore: ConnectionsStore,
        walletSessionService: any WalletSessionManageable,
        walletConnectorInteractor: any WalletConnectorInteractable,
        signingInteractor: any SigningRequestInteractable,
    ) {
        self.connectionsStore = connectionsStore
        self.walletConnectorInteractor = walletConnectorInteractor
        self.signingInteractor = signingInteractor
        self.walletSessionService = walletSessionService
    }

    public var allChains: [Primitives.Chain] {
        Config.shared.getWalletConnectConfig().chains.compactMap { Chain(rawValue: $0) }
    }

    public func getCurrentWallet() throws -> Wallet {
        try walletSessionService.getCurrentWallet()
    }

    public func getWallet(id: WalletId) throws -> Wallet {
        try walletSessionService.getWallet(walletId: id)
    }

    public func getChains(wallet: Wallet) -> [Primitives.Chain] {
        wallet.accounts.map(\.chain).asSet().intersection(allChains).asArray()
    }

    public func getAccounts(sessionId: String, chain: Primitives.Chain) throws -> [Primitives.Account] {
        let connection = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: connection.session)
        return connection.wallet.accounts.filter { $0.chain == chain }
    }

    public func getWallets(for proposal: Session.Proposal) throws -> [Wallet] {
        guard let requiredChains = proposal.supportedRequiredChains else { return [] }
        let optionalChains = proposal.supportedOptionalChains

        return try walletSessionService.getWallets()
            .filter {
                guard !$0.isViewOnly else { return false }

                let walletChains = $0.accounts.map(\.chain).filter { $0 != .bitcoin }.asSet()
                guard walletChains.isNotEmpty else { return false }

                if requiredChains.isNotEmpty {
                    return walletChains.isSuperset(of: requiredChains)
                }

                return optionalChains.isEmpty || walletChains.contains(where: optionalChains.contains)
            }
    }

    public func getEvents() -> [WalletConnectionEvents] {
        WalletConnectionEvents.allCases
    }

    public func getMethods() -> [WalletConnectionMethods] {
        WalletConnectionMethods.allCases
    }

    public func sessionApproval(payload: WCPairingProposal) async throws -> WalletId {
        try await walletConnectorInteractor.sessionApproval(payload: payload)
    }

    public func sessionReject(error: any Error) async {
        await walletConnectorInteractor.sessionReject(error: error)
    }

    public func signMessage(sessionId: String, chain: Chain, message: SignMessage, simulation: SimulationResult) async throws -> String {
        let session = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: session.session)
        let payload = SignMessagePayload(
            id: session.session.id,
            chain: chain,
            appMetadata: session.session.metadata.transactionAppMetadata,
            wallet: session.wallet,
            message: message,
            simulation: simulation,
        )
        return try await signingInteractor.signMessage(payload: payload)
    }

    public func updateSessions(sessions: [WalletConnectionSession]) throws {
        if sessions.isEmpty {
            _ = try? connectionsStore.deleteAll()
        } else {
            let newSessionIds = sessions.map(\.id).asSet()
            let sessionIds = try connectionsStore.getSessions().filter { $0.state == .active }.map(\.id).asSet()
            let deleteIds = sessionIds.subtracting(newSessionIds).asArray()

            _ = try? connectionsStore.delete(ids: deleteIds)

            for session in sessions {
                try? connectionsStore.updateConnectionSession(session)
            }
        }
    }

    public func sessionReject(id: String, error: any Error) async throws {
        _ = try connectionsStore.delete(ids: [id])
        await walletConnectorInteractor.sessionReject(error: error)
    }

    public func signTransaction(sessionId: String, chain: Chain, transaction: SignableTransaction, simulation: SimulationResult) async throws -> String {
        let session = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: session.session)
        let wallet = try getWallet(id: session.wallet.id)

        switch transaction {
        case .ethereum:
            throw AnyError("Not supported")
        case .solana, .sui, .ton, .tron:
            let transferData = try SigningTransferDataFactory.transferData(
                chain: chain,
                appMetadata: session.session.metadata.transactionAppMetadata,
                transaction: transaction,
                outputAction: .sign,
            )
            return try await signingInteractor.signTransaction(transferData: SigningTransferData(transferData: transferData, wallet: wallet, simulation: simulation))
        }
    }

    public func sendTransaction(sessionId: String, chain: Chain, transaction: SignableTransaction, simulation: SimulationResult) async throws -> String {
        let session = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: session.session)
        let wallet = try getWallet(id: session.wallet.id)

        let transferData = try SigningTransferDataFactory.transferData(
            chain: chain,
            appMetadata: session.session.metadata.transactionAppMetadata,
            transaction: transaction,
            outputAction: .send,
        )
        return try await signingInteractor.sendTransaction(transferData: SigningTransferData(transferData: transferData, wallet: wallet, simulation: simulation))
    }

    public func sendRawTransaction(sessionId: String, chain: Chain, transaction: String) async throws -> String {
        let session = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: session.session)
        let wallet = try getWallet(id: session.wallet.id)
        let transferData = SigningTransferDataFactory.encodedTransferData(
            chain: chain,
            appMetadata: session.session.metadata.transactionAppMetadata,
            transaction: transaction,
            outputType: .encodedTransaction,
            outputAction: .send,
        )
        return try await walletConnectorInteractor.sendRawTransaction(
            transferData: SigningTransferData(
                transferData: transferData,
                wallet: wallet,
                simulation: .empty,
            ),
        )
    }

    private func validate(chain: Chain, session: WalletConnectionSession) throws {
        if !session.chains.contains(chain) {
            throw WalletConnectorServiceError.unresolvedChainId(chain.rawValue)
        }
    }

    public func addConnection(connection: WalletConnection) throws {
        try connectionsStore.addConnection(connection)
    }
}

extension Session.Proposal {
    var supportedRequiredChains: Set<Chain>? {
        requiredNamespaces.fullySupportedChains
    }

    var supportedOptionalChains: Set<Chain> {
        optionalNamespaces?.supportedChains ?? []
    }
}

private extension [String: ProposalNamespace] {
    var fullySupportedChains: Set<Chain>? {
        let blockchains = values.flatMap { $0.chains ?? [] }
        let chains = blockchains.compactMap(\.chain)
        guard chains.count == blockchains.count else { return .none }
        return chains.asSet()
    }

    var supportedChains: Set<Chain> {
        values
            .flatMap { $0.chains ?? [] }
            .compactMap(\.chain)
            .asSet()
    }
}
