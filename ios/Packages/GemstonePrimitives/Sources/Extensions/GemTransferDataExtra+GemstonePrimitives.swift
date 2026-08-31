// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension GemTransferDataExtra {
    func map() throws -> TransferDataExtra {
        try TransferDataExtra(
            to: to,
            gasLimit: gasLimit.map { try BigInt.from(string: $0) },
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
    func map() throws -> GemTransferDataExtra {
        GemTransferDataExtra(
            to: to,
            gasLimit: gasLimit?.description,
            gasPrice: gasPrice?.map(),
            data: data,
            outputType: outputType.json(),
            outputAction: outputAction.json(),
            transactionType: transactionType.json(),
            approval: approval?.json(),
        )
    }
}




