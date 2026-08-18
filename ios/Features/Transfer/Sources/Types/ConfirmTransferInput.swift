// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents

public struct ConfirmTransferInput: Sendable {
    public let transactionData: TransactionData
    public let transferAmount: TransferAmountValidation
    public let feeAsset: Asset
    public let feeAssetBalance: Balance

    public init(
        transactionData: TransactionData,
        transferAmount: TransferAmountValidation,
        feeAsset: Asset,
        feeAssetBalance: Balance,
    ) {
        self.transactionData = transactionData
        self.transferAmount = transferAmount
        self.feeAsset = feeAsset
        self.feeAssetBalance = feeAssetBalance
    }
}
