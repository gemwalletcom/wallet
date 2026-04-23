// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Store

public struct TransactionStateProcessor: Sendable {
    private let transactionStore: TransactionStore
    private let postProcessingService: TransactionPostProcessingService

    public init(transactionStore: TransactionStore, postProcessingService: TransactionPostProcessingService) {
        self.transactionStore = transactionStore
        self.postProcessingService = postProcessingService
    }

    public func process(id: String, changes: TransactionChanges) async {
        do {
            let transactionId = try perform(changes: changes, to: id)

            guard changes.state.isCompleted,
                  let transactionWallet = try getTransactionWallet(transactionId: transactionId)
            else { return }

            try await postProcessingService.process(
                wallet: transactionWallet.wallet,
                transaction: transactionWallet.transaction,
            )
        } catch {
            debugLog("TransactionStateProcessor: \(id) failed: \(error)")
        }
    }
}

// MARK: - Private

private extension TransactionStateProcessor {
    func getTransactionWallet(transactionId: String) throws -> TransactionWallet? {
        try transactionStore.getTransactionWallet(id: transactionId)
    }

    func perform(changes: TransactionChanges, to id: String) throws -> String {
        let transactionId = try changes.changes.reduce(id) { id, change in
            switch change {
            case let .networkFee(fee):
                _ = try transactionStore.updateNetworkFee(transactionId: id, networkFee: fee.description)
                return id
            case let .blockNumber(block):
                _ = try transactionStore.updateBlockNumber(transactionId: id, block: block)
                return id
            case let .createdAt(date):
                _ = try transactionStore.updateCreatedAt(transactionId: id, date: date)
                return id
            case let .metadata(metadata):
                _ = try transactionStore.updateMetadata(transactionId: id, metadata: metadata)
                return id
            case let .hashChange(_, newHash):
                let chain = try TransactionId(id: id).chain
                let newId = Transaction.id(chain: chain, hash: newHash)
                try transactionStore.updateTransactionId(oldTransactionId: id, transactionId: newId, hash: newHash)
                return newId
            }
        }
        _ = try transactionStore.updateState(id: transactionId, state: changes.state)
        return transactionId
    }
}
