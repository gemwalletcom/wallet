// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

enum PaymentTransactionFactory {
    static func makePendingPayment(provider: PaymentProviderName, quote: PaymentQuote, merchant: PaymentMerchant, wallet: Wallet) throws -> Transaction {
        let assetId = quote.amount.assetId
        let account = try wallet.account(for: assetId.chain)
        guard let metadata = AnyCodableValue.encode(TransactionPaymentMetadata(paymentId: quote.paymentId, merchant: merchant, provider: provider)) else {
            throw AnyError("payment metadata is not encodable")
        }

        return Transaction(
            id: TransactionId(chain: assetId.chain, hash: quote.paymentId),
            assetId: assetId,
            from: account.address,
            to: .empty,
            contract: assetId.tokenId,
            type: .transfer,
            state: .pending,
            blockNumber: .none,
            sequence: .none,
            fee: .zero,
            feeAssetId: assetId.chain.assetId,
            value: quote.amount.value,
            memo: .none,
            direction: .outgoing,
            utxoInputs: .none,
            utxoOutputs: .none,
            metadata: metadata,
            createdAt: Date(),
        )
    }
}
