// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct DeepLinkTests {
    @Test
    func url() {
        #expect(DeepLink.asset(.mock()).url.absoluteString == DeepLinkMock.assetBitcoin)
        #expect(DeepLink.asset(.mockEthereumUSDT()).url.absoluteString == DeepLinkMock.assetEthereumToken)
        #expect(DeepLink.perpetuals.url.absoluteString == DeepLinkMock.perpetuals)
        #expect(DeepLink.rewards(code: "gemcoder").url.absoluteString == DeepLinkMock.rewards)
        #expect(DeepLink.gift(code: "giftcode123").url.absoluteString == DeepLinkMock.gift)
    }

    @Test
    func gemUrl() {
        #expect(DeepLink.perpetuals.gemUrl.absoluteString == DeepLinkMock.perpetualsGem)
        #expect(DeepLink.asset(.mock()).gemUrl.absoluteString == DeepLinkMock.assetBitcoinGem)
    }
}
