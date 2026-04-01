// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension BalanceViewModel {
    static func mock(
        asset: Asset = .mock(),
        balance: Balance = .mock(),
        formatter: ValueFormatter = .auto,
    ) -> BalanceViewModel {
        BalanceViewModel(
            asset: asset,
            balance: balance,
            formatter: formatter,
        )
    }
}
