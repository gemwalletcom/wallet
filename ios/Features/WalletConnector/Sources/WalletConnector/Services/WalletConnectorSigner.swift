// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import class Gemstone.Config
import enum Gemstone.GemServiceError
import protocol Gemstone.GemWalletConnectSigner
import struct Gemstone.GemWalletConnectSignRequest
import enum Gemstone.GemWalletConnectTransactionAction
import GemstonePrimitives
import GemstoneServices
import Primitives
import WalletConnectorService

public final class WalletConnectorSigner: WalletConnectorSignable, GemWalletConnectSigner {
    private let walletConnectorInteractor: any WalletConnectorInteractable
    private let walletSessionService: any WalletSessionManageable

    public init(
        walletSessionService: any WalletSessionManageable,
        walletConnectorInteractor: any WalletConnectorInteractable,
    ) {
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

    public func sessionApproval(payload: WCPairingProposal) async throws -> WalletId {
        try await walletConnectorInteractor.sessionApproval(payload: payload)
    }

    public func sessionReject(error: any Error) async {
        await walletConnectorInteractor.sessionReject(error: error)
    }

    public func sign(request: GemWalletConnectSignRequest) async throws -> String {
        let chain = try request.chain.map()
        let session = try WalletConnectionSession(request.session)
        let wallet = try Wallet(request.wallet)
        let simulation = try SimulationResult(request.simulation)

        switch request.payload {
        case let .message(message):
            let payload = SignMessagePayload(chain: chain, session: session, wallet: wallet, message: message, simulation: simulation)
            return try await interact { try await walletConnectorInteractor.signMessage(payload: payload) }
        case let .transaction(transaction, action):
            let transferData = try transferData(chain: chain, session: session, transaction: transaction.map(), action: action)
            let data = WCTransferData(transferData: transferData, wallet: wallet, simulation: simulation)
            return try await interact {
                switch action {
                case .sign: try await walletConnectorInteractor.signTransaction(transferData: data)
                case .send: try await walletConnectorInteractor.sendTransaction(transferData: data)
                }
            }
        }
    }

    private func interact(_ action: () async throws -> String) async throws -> String {
        do {
            return try await action()
        } catch ConnectionsError.userCancelled {
            throw GemServiceError.Cancelled
        }
    }

    private func transferData(
        chain: Primitives.Chain,
        session: WalletConnectionSession,
        transaction: WalletConnectorTransaction,
        action: GemWalletConnectTransactionAction,
    ) throws -> TransferData {
        switch transaction {
        case let .ethereum(transaction, kind):
            guard action == .send else {
                throw AnyError("Not supported")
            }
            return try ethereumTransferData(chain: chain, session: session, transaction: transaction, kind: kind)
        case let .solana(encodedTransaction, outputType, _),
             let .sui(encodedTransaction, outputType),
             let .ton(encodedTransaction, outputType),
             let .tron(encodedTransaction, outputType):
            return TransferData(
                type: .generic(
                    asset: chain.asset,
                    metadata: session.metadata,
                    extra: TransferDataExtra(
                        to: "",
                        data: Data(encodedTransaction.utf8),
                        outputType: outputType,
                        outputAction: action.outputAction,
                        transactionType: transaction.transactionType,
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

    private func ethereumTransferData(
        chain: Primitives.Chain,
        session: WalletConnectionSession,
        transaction: WCEthereumTransaction,
        kind: WalletConnectorEVMTransactionKind,
    ) throws -> TransferData {
        let value = try BigInt.fromHex(transaction.value ?? .zero)
        let gasLimit = (transaction.gasLimit ?? transaction.gas).flatMap { BigInt(hex: $0) }
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

        return TransferData(
            type: .generic(asset: chain.asset, metadata: session.metadata, extra: TransferDataExtra(
                to: transaction.to,
                gasLimit: gasLimit,
                gasPrice: gasPrice,
                data: transaction.data.map { Data(hex: $0) },
                transactionType: kind.transactionType,
                approval: kind.approvalData,
            )),
            recipientData: RecipientData(
                recipient: Recipient(name: .none, address: transaction.to, memo: .none),
                amount: .none,
            ),
            amount: .exact(value),
        )
    }
}

private extension GemWalletConnectTransactionAction {
    var outputAction: TransferDataOutputAction {
        switch self {
        case .sign: .sign
        case .send: .send
        }
    }
}
