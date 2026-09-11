// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.communityLinks
import enum Gemstone.LinkType
@testable import PrimitivesComponents
import Testing

struct DeepLinkViewModelTests {
    @Test
    func deepLinks() {
        #expect(DeepLinkViewModel.mock(.telegram)?.deepLink?.absoluteString == "tg://resolve?domain=gemwallet")
        #expect(DeepLinkViewModel.mock(.x)?.deepLink?.absoluteString == "twitter://user?screen_name=GemWallet")
        #expect(DeepLinkViewModel.mock(.youTube)?.deepLink?.absoluteString == "youtube://www.youtube.com/@gemwallet")
        #expect(DeepLinkViewModel.mock(.discord)?.deepLink?.absoluteString == "https://discord.gg/aWkq5sj7SY")
        #expect(DeepLinkViewModel.mock(.gitHub)?.deepLink?.absoluteString == "https://github.com/gemwalletcom")
    }
}

extension DeepLinkViewModel {
    static func mock(_ linkType: LinkType) -> DeepLinkViewModel? {
        communityLinks().first { $0.linkType == linkType }.map { DeepLinkViewModel($0) }
    }
}
