// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemSimulationValue
import GemstonePrimitives
import Localization
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct AssetValueHeaderTests {
    @Test
    func unlimitedTitle() {
        let model = GemSimulationValue(asset: Asset.mockEthereumUSDT().toGem(), value: .unlimited).valueHeader

        #expect(model.title == Localized.Simulation.Header.unlimitedAsset("USDT"))
        #expect(model.subtitle == nil)
    }

    @Test
    func formattedNumericTitle() {
        let model = GemSimulationValue(asset: Asset.mockEthereumUSDT().toGem(), value: .exact(value: BigUInt(1_000_000))).valueHeader

        #expect(model.title == "1 USDT")
    }
}
