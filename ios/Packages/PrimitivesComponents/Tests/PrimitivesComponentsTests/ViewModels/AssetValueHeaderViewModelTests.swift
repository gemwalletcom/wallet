// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Localization
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct AssetValueHeaderViewModelTests {
    @Test
    func unlimitedTitle() {
        let model = AssetValueHeaderViewModel(
            data: AssetValueHeaderData(asset: .mockEthereumUSDT(), value: .unlimited),
        )

        #expect(model.title == Localized.Simulation.Header.unlimitedAsset("USDT"))
        #expect(model.subtitle == nil)
    }

    @Test
    func formattedNumericTitle() {
        let model = AssetValueHeaderViewModel(
            data: AssetValueHeaderData(asset: .mockEthereumUSDT(), value: .exact(BigInt(1_000_000))),
        )

        #expect(model.title == "1 USDT")
    }
}
