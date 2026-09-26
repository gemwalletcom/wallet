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
        let model = GemSimulationValue(
            asset: Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20).toGem(),
            value: .unlimited,
            icon: .mock(),
        ).valueHeader

        #expect(model.title == Localized.Simulation.Header.unlimitedAsset("USDT"))
        #expect(model.subtitle == nil)
    }

    @Test
    func formattedNumericTitle() {
        let model = GemSimulationValue(
            asset: Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20).toGem(),
            value: .exact(value: BigUInt(1_000_000)),
            icon: .mock(),
        ).valueHeader

        #expect(model.title == "1 USDT")
    }
}
