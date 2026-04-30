// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import ChainService
import Foundation
import Primitives
import Store

struct TransactionStateProvider {
    private let transactionStore: TransactionStore
    private let chainServiceFactory: any ChainServiceFactorable

    init(
        transactionStore: TransactionStore,
        chainServiceFactory: any ChainServiceFactorable,
    ) {
        self.transactionStore = transactionStore
        self.chainServiceFactory = chainServiceFactory
    }

    func getState(for transaction: Transaction) async throws -> TransactionChanges {
        let chainService = chainServiceFactory.service(for: transaction.assetId.chain)
        return try await chainService.transactionState(
            for: TransactionStateRequest(
                id: transaction.id.hash,
                senderAddress: transaction.from,
                recipientAddress: transaction.to,
                block: Int.from(string: transaction.blockNumber ?? "0"),
                createdAt: transaction.createdAt,
            ),
        )
    }

    func updateStateChanges(_ stateChanges: TransactionChanges, for transaction: Transaction) async throws {
        debugLog("updateStateChanges for \(transaction.id), \(stateChanges)")

        try updateState(
            state: stateChanges.state,
            for: transaction,
        )

        for change in stateChanges.changes {
            let transactionId = transaction.id.identifier
            switch change {
            case let .networkFee(networkFee):
                _ = try transactionStore.updateNetworkFee(
                    transactionId: transaction.id.identifier,
                    networkFee: networkFee.description,
                )
            case let .hashChange(_, newHash):
                let newTransactionId = Primitives.Transaction.id(chain: transaction.assetId.chain, hash: newHash)
                try transactionStore.updateTransactionId(
                    oldTransactionId: transactionId,
                    transactionId: newTransactionId,
                    hash: newHash,
                )
            case let .blockNumber(block):
                _ = try transactionStore.updateBlockNumber(transactionId: transactionId, block: block)
            case let .createdAt(date):
                _ = try transactionStore.updateCreatedAt(transactionId: transactionId, date: date)
            case let .metadata(metadata):
                _ = try transactionStore.updateMetadata(transactionId: transactionId, metadata: metadata)
            }
        }
    }

    func updateState(state: TransactionState, for transaction: Transaction) throws {
        _ = try transactionStore.updateState(
            id: transaction.id.identifier,
            state: state,
        )
    }
}
