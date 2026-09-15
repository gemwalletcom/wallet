// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension AssetDataViewModel {
    static func mock(
        assetData: AssetData = .mock(),
        formatter: ValueFormatter = .short,
        currency: Currency = .usd,
    ) -> AssetDataViewModel {
        AssetDataViewModel(
            assetData: assetData,
            formatter: formatter,
            currency: currency,
        )
    }
}
