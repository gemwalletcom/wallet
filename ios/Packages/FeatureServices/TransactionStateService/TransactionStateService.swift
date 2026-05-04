// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Foundation
import Primitives
import Store

public struct TransactionStateService: Sendable {
    private let transactionStore: TransactionStore
    private let gatewayService: GatewayService
    private let postProcessingService: TransactionPostProcessingService

    public init(
        transactionStore: TransactionStore,
        gatewayService: GatewayService,
        postProcessingService: TransactionPostProcessingService,
    ) {
        self.transactionStore = transactionStore
        self.gatewayService = gatewayService
        self.postProcessingService = postProcessingService
    }

    func update(for transaction: Transaction) async -> JobStatus {
        do {
            let stateChanges = try await gatewayService.transactionStatus(
                chain: transaction.assetId.chain,
                request: TransactionStateRequest(
                    id: transaction.id.hash,
                    senderAddress: transaction.from,
                    block: Int.from(string: transaction.blockNumber ?? "0"),
                    createdAt: transaction.createdAt,
                    swapProvider: transaction.swapProvider.flatMap(SwapProvider.init(rawValue:)),
                ),
            )
            switch stateChanges.state {
            case .pending, .inTransit:
                return .retry
            case .confirmed, .reverted, .failed:
                try update(stateChanges, for: transaction)
                return .complete
            }
        } catch {
            debugLog("TransactionStateService: \(error)")
            return .retry
        }
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
    private func update(_ stateChanges: TransactionChanges, for transaction: Transaction) throws {
        let id = transaction.id.identifier
        _ = try transactionStore.updateState(id: id, state: stateChanges.state)
        for change in stateChanges.changes {
            switch change {
            case let .networkFee(fee):
                _ = try transactionStore.updateNetworkFee(transactionId: id, networkFee: fee.description)
            case let .hashChange(_, newHash):
                let newTransactionId = Primitives.Transaction.id(chain: transaction.assetId.chain, hash: newHash)
                try transactionStore.updateTransactionId(oldTransactionId: id, transactionId: newTransactionId, hash: newHash)
            case let .blockNumber(block):
                _ = try transactionStore.updateBlockNumber(transactionId: id, block: block)
            case let .createdAt(date):
                _ = try transactionStore.updateCreatedAt(transactionId: id, date: date)
            case let .metadata(metadata):
                _ = try transactionStore.updateMetadata(transactionId: id, metadata: metadata)
            }
        }
    }
}
