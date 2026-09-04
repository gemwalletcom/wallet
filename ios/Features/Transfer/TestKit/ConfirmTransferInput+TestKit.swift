// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemConfirmData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import struct Gemstone.GemTransactionLoadFee
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Transfer
import Validators
import struct Gemstone.GemTransferData

public extension ConfirmTransferInput {
    static func mock(
        confirmData: GemConfirmData = .mock(input: GemTransferData.mock().confirmInput(from: .mock())),
        fee: GemTransactionLoadFee = .mock(),
        transferAmount: TransferAmountValidation = .success(
            TransferAmount(value: BigInt(100), networkFee: BigInt(21000), useMaxAmount: false),
        ),
        feeAsset: Asset = .mock(),
    ) -> ConfirmTransferInput {
        ConfirmTransferInput(
            confirmData: confirmData,
            fee: fee,
            transferAmount: transferAmount,
            feeAsset: feeAsset,
        )
    }
}
