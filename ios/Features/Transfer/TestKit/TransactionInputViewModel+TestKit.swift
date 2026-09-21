// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemConfirmMetadata
import struct Gemstone.GemTransactionLoadFee
import struct Gemstone.GemTransferData
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Transfer

public extension TransactionInputViewModel {
    static func mock(
        data: GemTransferData = .mock(),
        fee: GemTransactionLoadFee? = nil,
        metaData: GemConfirmMetadata? = nil,
        transferAmount: TransferAmountValidation? = nil,
        feeAsset: Asset = .mock(),
        currency: String = Currency.usd.rawValue,
    ) -> TransactionInputViewModel {
        TransactionInputViewModel(
            data: data,
            fee: fee,
            metaData: metaData,
            transferAmount: transferAmount,
            feeAsset: feeAsset,
            currency: currency,
        )
    }
}
