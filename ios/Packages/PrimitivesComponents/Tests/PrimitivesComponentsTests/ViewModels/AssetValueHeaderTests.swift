// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemValueHeader
import GemstonePrimitives
import Localization
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct AssetValueHeaderTests {
    @Test
    func unlimitedTitle() {
        let model = GemValueHeader(icon: .asset(icon: .mock()), title: .unlimitedAsset(symbol: "USDT"), subtitle: nil, subtitleIcon: nil, actions: nil).valueHeader

        #expect(model.title == Localized.Simulation.Header.unlimitedAsset("USDT"))
        #expect(model.subtitle == nil)
        #expect(model.assetImage != nil)
    }
}
