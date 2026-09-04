// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemConfirmData
import GemstonePrimitives
import struct Gemstone.GemTransactionLoadFee
import Primitives
import PrimitivesComponents

public struct ConfirmTransferInput: Sendable {
    public let confirmData: GemConfirmData
    public let fee: GemTransactionLoadFee
    public let transferAmount: TransferAmountValidation
    public let feeAsset: Asset

    public init(
        confirmData: GemConfirmData,
        fee: GemTransactionLoadFee,
        transferAmount: TransferAmountValidation,
        feeAsset: Asset,
    ) {
        self.confirmData = confirmData
        self.fee = fee
        self.transferAmount = transferAmount
        self.feeAsset = feeAsset
    }
}
