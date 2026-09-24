// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension AssetDataViewModel {
    static func mock(
        assetData: AssetData = .mock(),
        currency: Currency = .usd,
    ) -> AssetDataViewModel {
        AssetDataViewModel(
            assetData: assetData,
            currency: currency,
        )
    }
}
