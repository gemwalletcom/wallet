// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents

public struct ConfirmTransferInput: Sendable {
    public let transactionData: TransactionData
    public let transferAmount: TransferAmountValidation

    public init(
        transactionData: TransactionData,
        transferAmount: TransferAmountValidation,
    ) {
        self.transactionData = transactionData
        self.transferAmount = transferAmount
    }
}
