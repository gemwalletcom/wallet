// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Transfer
import Validators

public extension ConfirmTransferInput {
    static func mock(
        transactionData: TransactionData = .mock(),
        transferAmount: TransferAmountValidation = .success(
            TransferAmount(value: BigInt(100), networkFee: BigInt(21000), useMaxAmount: false),
        ),
        feeAsset: Asset = .mock(),
    ) -> ConfirmTransferInput {
        ConfirmTransferInput(
            transactionData: transactionData,
            transferAmount: transferAmount,
            feeAsset: feeAsset,
        )
    }
}
