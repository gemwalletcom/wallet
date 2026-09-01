// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension SwapQuoteData {
    static func mock(
        approval: ApprovalData? = nil,
    ) -> SwapQuoteData {
        SwapQuoteData(
            to: "",
            dataType: .contract,
            value: "0",
            data: "",
            memo: nil,
            approval: approval,
            gasLimit: "",
        )
    }
}
