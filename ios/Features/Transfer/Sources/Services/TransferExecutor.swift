// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import enum Gemstone.GemConfirmError
import protocol Gemstone.GemConfirmServiceProtocol
import struct Gemstone.GemPendingTransactionInput
import struct Gemstone.GemSignedTransaction
import class Gemstone.GemTransferService
import GemstonePrimitives
import Primitives

public protocol TransferExecutable: Sendable {
    func execute(input: TransferConfirmationInput) async throws
}

public struct TransferExecutor: TransferExecutable {
    private static let ignoredAssetChains: Set<Chain> = [.hyperCore]

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
                try await record(input: input, hashes: broadcasted, transactions: transactions)
                throw AnyError(msg)
            }
            throw error
        }
        try await record(input: input, hashes: hashes, transactions: transactions)
    }

    private func record(input: TransferConfirmationInput, hashes: [String], transactions: [GemSignedTransaction]) async throws {
        for (index, hash) in hashes.enumerated() {
            debugLog("TransferExecutor broadcast response hash \(hash)")
            input.delegate?(.success(hash))
            let pending = try GemTransferService().pendingTransaction(input: GemPendingTransactionInput(
                sender: input.wallet.account(for: input.data.chain).address,
                transfer: input.data.gem,
                value: input.amount.value.description,
                transactionType: transactions[index].transactionType,
                hash: hash,
                fee: input.transactionData.fee.map(),
                networkFee: input.amount.networkFee.description,
                metadata: input.transactionData.metadata,
                simulation: input.simulation?.json(),
                transactionIndex: UInt32(index),
                transactionCount: UInt32(transactions.count),
            )).map { try Transaction($0) }
            guard let transaction = pending else { continue }
            try await transactionStateScheduler.addTransactions(wallet: input.wallet, transactions: [transaction])
            let assetIds = assetIdsToEnable(for: transaction)
            Task {
                do {
                    try await assetsEnabler.enableAssets(wallet: input.wallet, assetIds: assetIds, enabled: true)
                } catch {
                    debugLog("TransferExecutor post-transfer asset update error: \(error)")
                }
            }
        }
    }

    private func assetIdsToEnable(for transaction: Transaction) -> [AssetId] {
        transaction.assetIds.filter { !Self.ignoredAssetChains.contains($0.chain) }
    }
}
