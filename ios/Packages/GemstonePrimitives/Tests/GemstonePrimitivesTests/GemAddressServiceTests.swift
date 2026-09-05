// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAddressService
import GemstonePrimitives
import Primitives
import Testing

struct GemAddressServiceTests {
    @Test
    func formatsForTheChainWithTheShortStyleByDefault() {
        #expect(GemAddressService.shared.format(address: "0x12312321321312", chain: .ethereum) == "0x12312...21312")
        #expect(GemAddressService.shared.format(address: "0x12312321321312", chain: .aptos) == "0x1231...21312")
        #expect(GemAddressService.shared.format(address: "0x1231232221321312", chain: .ethereum, style: .full) == "0x1231232221321312")
    }
}
