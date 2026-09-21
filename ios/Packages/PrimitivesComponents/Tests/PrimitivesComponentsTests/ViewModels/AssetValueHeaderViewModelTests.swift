// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemSimulationValue
import GemstonePrimitives
import Localization
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct AssetValueHeaderViewModelTests {
    @Test
    func unlimitedTitle() {
        let model = AssetValueHeaderViewModel(
            data: GemSimulationValue(asset: Asset.mockEthereumUSDT().toGem(), value: .unlimited),
        )

        #expect(model.title == Localized.Simulation.Header.unlimitedAsset("USDT"))
        #expect(model.subtitle == nil)
    }

    @Test
    func formattedNumericTitle() {
        let model = AssetValueHeaderViewModel(
            data: GemSimulationValue(asset: Asset.mockEthereumUSDT().toGem(), value: .exact(value: BigUInt(1_000_000))),
        )

        #expect(model.title == "1 USDT")
    }
}
