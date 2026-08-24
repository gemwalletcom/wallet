// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import GemstonePrimitives
import Primitives

public struct PaymentTransferDataFactory: Sendable {
    public static func payment(quoteData: PaymentQuoteData, merchant: PaymentMerchant) throws -> TransferData {
        switch quoteData.action {
        case let .send(action):
            guard let value = BigInt(action.value) else {
                throw AnyError("Unsupported payment value: \(action.value)")
            }
            return TransferData(
                type: .payment(
                    asset: action.chain.asset,
                    payment: PaymentData(quote: quoteData.quote, merchant: merchant),
                    extra: TransferDataExtra(
                        to: action.recipient,
                        data: Data(fromHex: action.data),
                        transactionType: .transfer,
                    ),
                ),
                recipientData: RecipientData(
                    recipient: Recipient(name: merchant.name, address: action.recipient, memo: .none),
                    amount: .none,
                ),
                amount: .exact(value),
            )
        }
    }
}
