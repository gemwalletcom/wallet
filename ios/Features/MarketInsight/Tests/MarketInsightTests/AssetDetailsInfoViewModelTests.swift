// Copyright (c). Gem Wallet. All rights reserved.

import Localization
@testable import MarketInsight
import Primitives
import PrimitivesTestKit
import Testing

struct AssetDetailsInfoViewModelTests {
    @Test
    func marketValuesKeepFDVUnderMarketCapAndSeparateContract() {
        let model = AssetDetailsInfoViewModel(
            priceData: PriceData(
                asset: .mockEthereumUSDT(),
                price: .none,
                priceAlerts: [],
                market: .mock(),
                links: [],
            ),
        )

        #expect(model.marketValues.map(\.title) == [
            Localized.Asset.marketCap,
            Localized.Info.FullyDilutedValuation.title,
            Localized.Asset.tradingVolume,
        ])
        #expect(model.contractValues.map(\.title) == [Localized.Asset.contract])
    }
}
