// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmMetadata
import BigInt
import Foundation
import GemstonePrimitivesTestKit
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Transfer
import struct Gemstone.GemTransferData

public extension TransactionInputViewModel {
    static func mock(
        data: GemTransferData = .mock(),
        fee: Fee? = nil,
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
