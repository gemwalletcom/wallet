// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents

public struct ConfirmTransferInput: Sendable {
    public let transactionData: TransactionData
    public let feeRates: [FeeRate]
    public let transferAmount: TransferAmountValidation

    public init(
        transactionData: TransactionData,
        feeRates: [FeeRate],
        transferAmount: TransferAmountValidation,
    ) {
        self.transactionData = transactionData
        self.feeRates = feeRates
        self.transferAmount = transferAmount
    }
}
