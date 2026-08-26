// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import Blockchain
import Foundation
import enum Gemstone.GemConfirmError
import protocol Gemstone.GemConfirmServiceProtocol
import struct Gemstone.GemSignedTransaction
import GemstonePrimitives
import Primitives
import Signer
import TransactionStateService

public protocol TransferExecutable: Sendable {
    func execute(input: TransferConfirmationInput) async throws
}

public struct TransferExecutor: TransferExecutable {
    private static let ignoredTransactionTypes: Set<TransactionType> = [.perpetualModifyPosition]
    private static let ignoredAssetChains: Set<Chain> = [.hyperCore]
    private static let hyperCoreOrderIdPrefix = "order:"

    private let signer: any TransactionSigning
    private let confirmService: any GemConfirmServiceProtocol
    private let assetsEnabler: any AssetsEnabler
    private let transactionStateScheduler: TransactionStateScheduler

    public init(
        signer: any TransactionSigning,
        confirmService: any GemConfirmServiceProtocol,
        assetsEnabler: any AssetsEnabler,
        transactionStateScheduler: TransactionStateScheduler,
    ) {
        self.signer = signer
        self.confirmService = confirmService
        self.assetsEnabler = assetsEnabler
        self.transactionStateScheduler = transactionStateScheduler
    }

    public func execute(input: TransferConfirmationInput) async throws {
        let signedTransactions = try await signer.sign(
            transfer: input.data,
            transactionData: input.transactionData,
            amount: input.amount,
            wallet: input.wallet,
        )

        switch input.data.type.outputAction {
        case .sign:
            for signedTransaction in signedTransactions {
                input.delegate?(.success(signedTransaction.data))
            }
        case .send:
            try await broadcast(input: input, transactions: signedTransactions)
        }
    }
}

// MARK: - Private

extension TransferExecutor {
    private func broadcast(input: TransferConfirmationInput, transactions: [GemSignedTransaction]) async throws {
        let hashes: [String]
        do {
            hashes = try await confirmService.broadcast(inputType: input.data.type.map(), transactions: transactions)
        } catch let error as GemConfirmError {
            if case let .Broadcast(broadcasted, msg) = error {
                try record(input: input, hashes: broadcasted, transactions: transactions)
                throw AnyError(msg)
            }
            throw error
        }
        try record(input: input, hashes: hashes, transactions: transactions)
    }

    private func record(input: TransferConfirmationInput, hashes: [String], transactions: [GemSignedTransaction]) throws {
        for (index, hash) in hashes.enumerated() {
            debugLog("TransferExecutor broadcast response hash \(hash)")

            input.delegate?(.success(hash))

            let transaction = try TransactionFactory.makePendingTransaction(
                wallet: input.wallet,
                transferData: input.data,
                transactionData: input.transactionData,
                amount: input.amount,
                hash: hash,
                transactionType: Primitives.TransactionType(transactions[index].transactionType),
                simulation: input.simulation,
            )
            let assetIds = assetIdsToEnable(for: transaction)
            let pending = pendingTransactions(
                for: transaction,
                transferData: input.data,
                transactionIndex: index,
                totalTransactions: transactions.count,
            )

            try transactionStateScheduler.addTransactions(wallet: input.wallet, transactions: pending)
            Task {
                do {
                    try await assetsEnabler.enableAssets(wallet: input.wallet, assetIds: assetIds, enabled: true)
                } catch {
                    debugLog("TransferExecutor post-transfer asset update error: \(error)")
                }
            }
        }
    }

    private func pendingTransactions(
        for transaction: Transaction,
        transferData: TransferData,
        transactionIndex: Int,
        totalTransactions: Int,
    ) -> [Transaction] {
        guard !Self.ignoredTransactionTypes.contains(transaction.type) else {
            return []
        }

        switch transaction.assetId.chain {
        case .hyperCore:
            switch transferData.type {
            case .stake where transactionIndex < totalTransactions - 1:
                return []
            case .perpetual where !transaction.id.hash.hasPrefix(Self.hyperCoreOrderIdPrefix):
                return []
            case let .swap(_, toAsset, data)
                where toAsset.chain == .hyperCore
                && data.quote.providerData.provider == .hyperliquid
                && transactionIndex < totalTransactions - 1:
                return []
            case .stake, .perpetual, .transfer, .deposit, .withdrawal, .transferNft, .swap, .tokenApprove, .generic, .account, .earn:
                break
            }
        default:
            break
        }

        return [transaction]
    }

    private func assetIdsToEnable(for transaction: Transaction) -> [AssetId] {
        transaction.assetIds.filter { !Self.ignoredAssetChains.contains($0.chain) }
    }
}
