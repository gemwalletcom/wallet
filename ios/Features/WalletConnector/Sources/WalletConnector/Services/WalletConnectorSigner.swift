// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.Account
import typealias Gemstone.Chain
import class Gemstone.Config
import enum Gemstone.GemServiceError
import protocol Gemstone.GemWalletConnectSigner
import struct Gemstone.SignMessage
import typealias Gemstone.SimulationResult
import enum Gemstone.WalletConnectTransaction
import GemstonePrimitives
import GemstoneServices
import Preferences
import Primitives
import Store
import WalletConnectorService

public final class WalletConnectorSigner: WalletConnectorSignable, GemWalletConnectSigner {
    private let connectionsStore: ConnectionsStore
    private let walletConnectorInteractor: any WalletConnectorInteractable
    private let walletSessionService: any WalletSessionManageable

    public init(
        connectionsStore: ConnectionsStore,
        walletSessionService: any WalletSessionManageable,
        walletConnectorInteractor: any WalletConnectorInteractable,
    ) {
        self.connectionsStore = connectionsStore
        self.walletConnectorInteractor = walletConnectorInteractor
        self.walletSessionService = walletSessionService
    }

    public var allChains: [Primitives.Chain] {
        Config.shared.getWalletConnectConfig().chains.compactMap { Primitives.Chain(rawValue: $0) }
    }

    public func getCurrentWallet() throws -> Wallet {
        try walletSessionService.getCurrentWallet()
    }

    public func getWallet(id: WalletId) throws -> Wallet {
        try walletSessionService.getWallet(walletId: id)
    }

    public func getWallets() throws -> [Wallet] {
        try walletSessionService.getWallets()
    }

    public func getAccounts(sessionId: String, chain: Gemstone.Chain) throws -> [Gemstone.Account] {
        let chain = try resolve(chain: chain)
        let connection = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: connection.session)
        return connection.wallet.accounts.filter { $0.chain == chain }.map { $0.mapToGem() }
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

    public func signMessage(sessionId: String, chain: Gemstone.Chain, message: SignMessage, simulation: Gemstone.SimulationResult) async throws -> String {
        let chain = try resolve(chain: chain)
        let session = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: session.session)

        let payload = try SignMessagePayload(
            chain: chain,
            session: session.session,
            wallet: session.wallet,
            message: message,
            simulation: Primitives.SimulationResult(simulation),
        )
        return try await interact { try await walletConnectorInteractor.signMessage(payload: payload) }
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

    public func signTransaction(sessionId: String, chain: Gemstone.Chain, transaction: WalletConnectTransaction, simulation: Gemstone.SimulationResult) async throws -> String {
        let chain = try resolve(chain: chain)
        let session = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: session.session)
        let wallet = try getWallet(id: session.wallet.id)
        let simulation = try Primitives.SimulationResult(simulation)
        let transaction = try transaction.map()
        let transactionType = transaction.transactionType

        switch transaction {
        case .ethereum:
            throw AnyError("Not supported")
        case let .solana(transaction, outputType, _),
             let .sui(transaction, outputType),
             let .ton(transaction, outputType),
             let .tron(transaction, outputType):
            let transferData = transferData(
                chain: chain,
                session: session.session,
                transaction: transaction,
                outputType: outputType,
                outputAction: .sign,
                transactionType: transactionType,
            )
            return try await interact { try await walletConnectorInteractor.signTransaction(transferData: WCTransferData(transferData: transferData, wallet: wallet, simulation: simulation)) }
        }
    }

    public func sendTransaction(sessionId: String, chain: Gemstone.Chain, transaction: WalletConnectTransaction, simulation: Gemstone.SimulationResult) async throws -> String {
        let chain = try resolve(chain: chain)
        let session = try connectionsStore.getConnection(id: sessionId)
        try validate(chain: chain, session: session.session)
        let wallet = try getWallet(id: session.wallet.id)
        let simulation = try Primitives.SimulationResult(simulation)
        let transaction = try transaction.map()
        let transactionType = transaction.transactionType

        switch transaction {
        case let .ethereum(transaction, kind):
            let address = transaction.to
            let value = try BigInt.fromHex(transaction.value ?? .zero)
            let gasLimit: BigInt? = {
                if let value = transaction.gasLimit {
                    return BigInt(hex: value)
                } else if let gas = transaction.gas {
                    return BigInt(hex: gas)
                }
                return .none
            }()

            let gasPrice: GasPriceType? = {
                if let maxFeePerGas = transaction.maxFeePerGas,
                   let maxPriorityFeePerGas = transaction.maxPriorityFeePerGas,
                   let maxFeePerGasBigInt = BigInt(hex: maxFeePerGas),
                   let maxPriorityFeePerGasBigInt = BigInt(hex: maxPriorityFeePerGas)
                {
                    return .eip1559(gasPrice: maxFeePerGasBigInt, priorityFee: maxPriorityFeePerGasBigInt)
                }
                return .none
            }()
            let data: Data? = {
                if let data = transaction.data {
                    return Data(hex: data)
                }
                return .none
            }()

            let transferData = TransferData(
                type: .generic(asset: chain.asset, metadata: session.session.metadata, extra: TransferDataExtra(
                    to: address,
                    gasLimit: gasLimit,
                    gasPrice: gasPrice,
                    data: data,
                    transactionType: kind.transactionType,
                    approval: kind.approvalData,
                )),
                recipientData: RecipientData(
                    recipient: Recipient(name: .none, address: address, memo: .none),
                    amount: .none,
                ),
                amount: .exact(value),
            )

            return try await interact { try await walletConnectorInteractor.sendTransaction(transferData: WCTransferData(transferData: transferData, wallet: wallet, simulation: simulation)) }
        case let .solana(transaction, outputType, _),
             let .sui(transaction, outputType),
             let .ton(transaction, outputType),
             let .tron(transaction, outputType):
            let transferData = transferData(
                chain: chain,
                session: session.session,
                transaction: transaction,
                outputType: outputType,
                outputAction: .send,
                transactionType: transactionType,
            )
            return try await interact { try await walletConnectorInteractor.sendTransaction(transferData: WCTransferData(transferData: transferData, wallet: wallet, simulation: simulation)) }
        }
    }

    private func resolve(chain: Gemstone.Chain) throws -> Primitives.Chain {
        guard let chain = Primitives.Chain(rawValue: chain) else {
            throw WalletConnectorServiceError.unresolvedChainId(chain)
        }
        return chain
    }

    private func interact(_ action: () async throws -> String) async throws -> String {
        do {
            return try await action()
        } catch ConnectionsError.userCancelled {
            throw GemServiceError.Cancelled
        }
    }

    private func validate(chain: Primitives.Chain, session: WalletConnectionSession) throws {
        if !session.chains.contains(chain) {
            throw WalletConnectorServiceError.unresolvedChainId(chain.rawValue)
        }
    }

    public func addConnection(connection: WalletConnection) throws {
        try connectionsStore.addConnection(connection)
    }

    private func transferData(
        chain: Primitives.Chain,
        session: WalletConnectionSession,
        transaction: String,
        outputType: TransferDataOutputType,
        outputAction: TransferDataOutputAction,
        transactionType: TransactionType,
    ) -> TransferData {
        TransferData(
            type: .generic(
                asset: chain.asset,
                metadata: session.metadata,
                extra: TransferDataExtra(
                    to: "",
                    data: Data(transaction.utf8),
                    outputType: outputType,
                    outputAction: outputAction,
                    transactionType: transactionType,
                ),
            ),
            recipientData: RecipientData(
                recipient: Recipient(name: nil, address: "", memo: nil),
                amount: nil,
            ),
            amount: .exact(.zero),
        )
    }
}
