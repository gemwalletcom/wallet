// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public struct ConfirmTransferInput: Sendable {
    public let transactionData: TransactionData
    public let transferAmount: TransferAmountValidation
    public let feeAsset: Asset

    public init(
        transactionData: TransactionData,
        transferAmount: TransferAmountValidation,
        feeAsset: Asset,
    ) {
        self.transactionData = transactionData
        self.transferAmount = transferAmount
        self.feeAsset = feeAsset
    }
}
