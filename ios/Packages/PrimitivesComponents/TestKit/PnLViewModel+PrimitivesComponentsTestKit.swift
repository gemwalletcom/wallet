// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import GemstonePrimitives
import PrimitivesComponents

public extension PnLViewModel {
    static func mock(
        pnl: Double? = .zero,
        marginAmount: Double = .zero,
    ) -> PnLViewModel {
        PnLViewModel(
            pnl: pnl,
            marginAmount: marginAmount,
            currencyFormatter: CurrencyFormatter.usd,
            percentageStyle: .signed,
        )
    }
}
