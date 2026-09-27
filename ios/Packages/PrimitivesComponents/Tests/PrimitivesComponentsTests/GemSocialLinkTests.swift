// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSocialLink
import GemstonePrimitivesTestKit
@testable import PrimitivesComponents
import Testing

struct GemSocialLinkTests {
    @Test
    func deepLinks() {
        #expect(GemSocialLink.mock(linkType: .telegram, url: "https://t.me/gemwallet").deepLink?.absoluteString == "tg://resolve?domain=gemwallet")
        #expect(GemSocialLink.mock(linkType: .x, url: "https://x.com/GemWallet").deepLink?.absoluteString == "twitter://user?screen_name=GemWallet")
        #expect(GemSocialLink.mock(linkType: .youTube, url: "https://www.youtube.com/@gemwallet").deepLink?.absoluteString == "youtube://www.youtube.com/@gemwallet")
        #expect(GemSocialLink.mock(linkType: .discord, url: "https://discord.gg/aWkq5sj7SY").deepLink?.absoluteString == "https://discord.gg/aWkq5sj7SY")
        #expect(GemSocialLink.mock(linkType: .gitHub, url: "https://github.com/gemwalletcom").deepLink?.absoluteString == "https://github.com/gemwalletcom")
        #expect(GemSocialLink.mock(linkType: .website, url: "https://example.com").deepLink == nil)
    }
}
