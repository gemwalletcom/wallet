// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Transfer

public extension TransactionInputViewModel {
    static func mock(
        data: TransferData = .mock(),
        transactionData: TransactionData? = nil,
        metaData: TransferDataMetadata? = nil,
        transferAmount: TransferAmountValidation? = nil,
    ) -> TransactionInputViewModel {
        TransactionInputViewModel(
            data: data,
            transactionData: transactionData,
            metaData: metaData,
            transferAmount: transferAmount,
        )
    }
}
