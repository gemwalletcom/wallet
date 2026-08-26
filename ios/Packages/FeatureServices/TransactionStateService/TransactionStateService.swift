// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Foundation
import func Gemstone.transactionTimeoutMs
import Primitives
import Store

protocol TransactionStatusServiceable: Sendable {
    func transactionUpdate(_ transaction: Transaction) async throws -> TransactionChanges
}

extension GatewayService: TransactionStatusServiceable {}

struct TransactionStateUpdateResult {
    let transactionId: TransactionId
    let status: JobStatus
}

private struct LocalTransactionRecord {
    let transactionId: TransactionId
    let state: TransactionState
}

public struct TransactionStateService: Sendable {
    private let transactionStore: TransactionStore
    private let postProcessingService: TransactionPostProcessingService
    private let statusService: any TransactionStatusServiceable

    public init(
        transactionStore: TransactionStore,
        gatewayService: GatewayService,
        postProcessingService: TransactionPostProcessingService,
    ) {
        self.init(
            transactionStore: transactionStore,
            postProcessingService: postProcessingService,
            statusService: gatewayService,
        )
    }

    init(
        transactionStore: TransactionStore,
        postProcessingService: TransactionPostProcessingService,
        statusService: any TransactionStatusServiceable,
    ) {
        self.transactionStore = transactionStore
        self.postProcessingService = postProcessingService
        self.statusService = statusService
    }

    func update(for transaction: Transaction) async -> TransactionStateUpdateResult {
        do {
            let stateChanges = try await statusService.transactionUpdate(transaction)
            return try saveStateChanges(stateChanges, for: transaction)
        } catch {
            // Gemstone applies the timeout when a status lookup succeeds. A lookup that
            // keeps failing never reaches that, so apply the same rule here rather than
            // retrying forever.
            if hasTimedOut(transaction), let result = try? saveStateChanges(TransactionChanges(state: .failed), for: transaction) {
                return result
            }
            return TransactionStateUpdateResult(
                transactionId: transaction.id,
                status: .retry(error: String(describing: error)),
            )
        }
    }

    private func hasTimedOut(_ transaction: Transaction) -> Bool {
        guard !transaction.state.isCompleted else { return false }
        let destinationChain: Primitives.Chain? = transaction.state == .inTransit
            ? transaction.metadata?.decode(TransactionSwapMetadata.self)?.toAsset.chain
            : nil
        let timeoutMs = transactionTimeoutMs(
            chain: transaction.assetId.chain.rawValue,
            destinationChain: destinationChain?.rawValue,
        )
        return Date().timeIntervalSince(transaction.createdAt) * 1000 > Double(timeoutMs)
    }

    func transactionWallet(walletId: WalletId, transactionId: TransactionId) throws -> TransactionWallet? {
        try transactionStore.getTransactionWallet(walletId: walletId, transactionId: transactionId)
    }

    func process(_ transactionWallet: TransactionWallet) async throws {
        try await postProcessingService.process(
            wallet: transactionWallet.wallet,
            transaction: transactionWallet.transaction,
        )
    }

    func updateBalances(_ transactionWallet: TransactionWallet) async {
        await postProcessingService.updateBalances(
            wallet: transactionWallet.wallet,
            transaction: transactionWallet.transaction,
        )
    }
}

// MARK: - Private

extension TransactionStateService {

    private func saveStateChanges(_ stateChanges: TransactionChanges, for transaction: Transaction) throws -> TransactionStateUpdateResult {
        guard stateChanges.state != transaction.state || !stateChanges.changes.isEmpty else {
            return TransactionStateUpdateResult(
                transactionId: transaction.id,
                status: transaction.state.isCompleted ? .complete : .retry(),
            )
        }

        let localTransaction = try localTransactionRecord(for: transaction, changes: stateChanges.changes)
        let nextState = try updateStateIfNeeded(
            transactionId: localTransaction.transactionId,
            oldState: localTransaction.state,
            newState: stateChanges.state,
        )
        try updateTransactionFields(stateChanges.changes, transactionId: localTransaction.transactionId)

        return TransactionStateUpdateResult(
            transactionId: localTransaction.transactionId,
            status: nextState.isCompleted ? .complete : .retry(),
        )
    }

    private func localTransactionRecord(for transaction: Transaction, changes: [TransactionChange]) throws -> LocalTransactionRecord {
        try changes.reduce(
            LocalTransactionRecord(
                transactionId: transaction.id,
                state: transaction.state,
            ),
        ) { localTransaction, change in
            guard case let .hashChange(_, newHash) = change else {
                return localTransaction
            }
            let newTransactionId = TransactionId(chain: transaction.assetId.chain, hash: newHash)
            let state = try transactionStore.updateTransactionId(
                oldTransactionId: localTransaction.transactionId,
                transactionId: newTransactionId,
                hash: newHash,
            ) ?? localTransaction.state
            return LocalTransactionRecord(
                transactionId: newTransactionId,
                state: state,
            )
        }
    }

    // Gemstone merges against the transaction it was handed. A hash change resolves
    // to a different local record, so guard that one from being walked backwards too.
    private func updateStateIfNeeded(transactionId: TransactionId, oldState: TransactionState, newState: TransactionState) throws -> TransactionState {
        let nextState = oldState == .pending || newState.isCompleted ? newState : oldState
        if nextState != oldState {
            _ = try transactionStore.updateState(id: transactionId, state: nextState)
        }
        return nextState
    }

    private func updateTransactionFields(_ changes: [TransactionChange], transactionId: TransactionId) throws {
        try changes.forEach { change in
            switch change {
            case let .networkFee(fee):
                _ = try transactionStore.updateNetworkFee(transactionId: transactionId, networkFee: fee.description)
            case .hashChange:
                break
            case let .blockNumber(block):
                _ = try transactionStore.updateBlockNumber(transactionId: transactionId, block: block)
            case let .createdAt(date):
                _ = try transactionStore.updateCreatedAt(transactionId: transactionId, date: date)
            case let .metadata(metadata):
                _ = try transactionStore.updateMetadata(transactionId: transactionId, metadata: metadata)
            case let .confirmationEtaSeconds(seconds):
                _ = try transactionStore.updateConfirmationEtaSeconds(transactionId: transactionId, seconds: seconds)
            }
        }
    }

}
