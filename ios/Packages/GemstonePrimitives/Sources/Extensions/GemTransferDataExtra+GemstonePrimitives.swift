// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemTransferDataExtra {
    func map() throws -> TransferDataExtra {
        try TransferDataExtra(
            to: to,
            gasLimit: gasLimit,
            gasPrice: gasPrice?.map(),
            data: data,
            outputType: Primitives.TransferDataOutputType(outputType),
            outputAction: Primitives.TransferDataOutputAction(outputAction),
            transactionType: Primitives.TransactionType(transactionType),
            approval: approval.map { try Primitives.ApprovalData($0) },
        )
    }
}

public extension TransferDataExtra {
    func map() -> GemTransferDataExtra {
        GemTransferDataExtra(
            to: to,
            gasLimit: gasLimit,
            gasPrice: gasPrice?.map(),
            data: data,
            outputType: outputType.json(),
            outputAction: outputAction.json(),
            transactionType: transactionType.json(),
            approval: approval?.json(),
        )
    }
}




