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
            outputType: outputType.map(),
            outputAction: outputAction.map(),
        )
    }
}

public extension TransferDataExtra {
    func map() -> GemTransferDataExtra {
        GemTransferDataExtra(
            to: to,
            gasLimit: gasLimit?.description,
            gasPrice: gasPrice?.map(),
            data: data,
            outputType: outputType.map(),
            outputAction: outputAction.map(),
        )
    }
}
