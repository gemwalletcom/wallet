// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension Primitives.SwapQuoteData {
    static func mock() -> Primitives.SwapQuoteData {
        Primitives.SwapQuoteData(
            to: "0x",
            dataType: .contract,
            value: "0",
            data: "0x",
            memo: nil,
            approval: .mock(),
            gasLimit: "210000",
        )
    }
}
