// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemConfirmData
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public struct ConfirmTransferInput: Sendable {
    public let confirmData: GemConfirmData
    public let fee: Fee
    public let transferAmount: TransferAmountValidation
    public let feeAsset: Asset

    public init(
        confirmData: GemConfirmData,
        fee: Fee,
        transferAmount: TransferAmountValidation,
        feeAsset: Asset,
    ) {
        self.confirmData = confirmData
        self.fee = fee
        self.transferAmount = transferAmount
        self.feeAsset = feeAsset
    }
}
