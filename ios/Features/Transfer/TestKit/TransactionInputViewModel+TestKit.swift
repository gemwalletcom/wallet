// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Transfer

public extension TransactionInputViewModel {
    static func mock(
        data: TransferData = .mock(),
        transactionData: TransactionData? = nil,
        metaData: TransferDataMetadata? = nil,
        transferAmount: TransferAmountValidation? = nil,
        feeAsset: Asset = .mock(),
        currency: String = Currency.usd.rawValue,
    ) -> TransactionInputViewModel {
        TransactionInputViewModel(
            data: data,
            transactionData: transactionData,
            metaData: metaData,
            transferAmount: transferAmount,
            feeAsset: feeAsset,
            currency: currency,
        )
    }
}
