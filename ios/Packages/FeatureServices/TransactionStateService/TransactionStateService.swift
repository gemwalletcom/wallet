// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Foundation
import Primitives
import Store

protocol TransactionStatusServiceable: Sendable {
    func transactionStatus(chain: Primitives.Chain, request: TransactionStateRequest) async throws -> TransactionChanges
    func transactionSwapStatus(chain: Primitives.Chain, request: TransactionSwapStateRequest) async throws -> TransactionChanges
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
            let stateChanges = try await fetchStateChanges(for: transaction)
            return try saveStateChanges(stateChanges, for: transaction)
        } catch {
            debugLog("TransactionStateService: \(error)")
            return TransactionStateUpdateResult(
                transactionId: transaction.id,
                status: .retry,
            )
        }
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
}

// MARK: - Private

extension TransactionStateService {
    private func fetchStateChanges(for transaction: Transaction) async throws -> TransactionChanges {
        let request = transactionStateRequest(for: transaction)
        if let swapRequest = transactionSwapStateRequest(for: transaction, transactionRequest: request) {
            return try await statusService.transactionSwapStatus(
                chain: transaction.assetId.chain,
                request: swapRequest,
            )
        }
        if transaction.type == .swap, transaction.state == .inTransit {
            return TransactionChanges(state: transaction.state)
        }
        return try await statusService.transactionStatus(
            chain: transaction.assetId.chain,
            request: request,
        )
    }

    private func transactionStateRequest(for transaction: Transaction) -> TransactionStateRequest {
        TransactionStateRequest(
            id: transaction.id.hash,
            senderAddress: transaction.from,
            createdAt: transaction.createdAt,
            blockNumber: transaction.blockNumber.flatMap(UInt64.init) ?? 0,
        )
    }

    private func transactionSwapStateRequest(
        for transaction: Transaction,
        transactionRequest: TransactionStateRequest,
    ) -> TransactionSwapStateRequest? {
        guard
            let swapMetadata = transaction.metadata?.decode(TransactionSwapMetadata.self),
            let swapProvider = swapMetadata.provider.flatMap(SwapProvider.init(rawValue:))
        else {
            return nil
        }
        return TransactionSwapStateRequest(
            transaction: transactionRequest,
            state: transaction.state,
            swapProvider: swapProvider,
            destinationChain: swapMetadata.toAsset.chain,
        )
    }

    private func saveStateChanges(_ stateChanges: TransactionChanges, for transaction: Transaction) throws -> TransactionStateUpdateResult {
        guard stateChanges.state != transaction.state || !stateChanges.changes.isEmpty else {
            return TransactionStateUpdateResult(
                transactionId: transaction.id,
                status: transaction.state.isCompleted ? .complete : .retry,
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
            status: nextState.isCompleted ? .complete : .retry,
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

    private func updateStateIfNeeded(transactionId: TransactionId, oldState: TransactionState, newState: TransactionState) throws -> TransactionState {
        let nextState = nextTransactionState(oldState: oldState, newState: newState)
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
            }
        }
    }

    private func nextTransactionState(oldState: TransactionState, newState: TransactionState) -> TransactionState {
        oldState == .pending || newState.isCompleted ? newState : oldState
    }
}
