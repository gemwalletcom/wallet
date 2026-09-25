// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSocialLink
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import Testing

struct GemSocialLinkTests {
    @Test
    func deepLinks() {
        #expect(GemSocialLink.mock(.telegram).deepLink?.absoluteString == "tg://resolve?domain=gemwallet")
        #expect(GemSocialLink.mock(.x).deepLink?.absoluteString == "twitter://user?screen_name=GemWallet")
        #expect(GemSocialLink.mock(.youTube).deepLink?.absoluteString == "youtube://www.youtube.com/@gemwallet")
        #expect(GemSocialLink.mock(.discord).deepLink?.absoluteString == "https://discord.gg/aWkq5sj7SY")
        #expect(GemSocialLink.mock(.gitHub).deepLink?.absoluteString == "https://github.com/gemwalletcom")
        #expect(GemSocialLink.mock(.website).deepLink == nil)
    }
}
