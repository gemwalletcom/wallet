// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemConfirmData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Transfer
import Validators

public extension ConfirmTransferInput {
    static func mock(
        confirmData: GemConfirmData = .mock(input: TransferData.mock().confirmInput(from: .mock())),
        fee: Fee = Fee(fee: 1, gasPriceType: .regular(gasPrice: 1), gasLimit: 1, feeAssetId: Asset.mock().id),
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
