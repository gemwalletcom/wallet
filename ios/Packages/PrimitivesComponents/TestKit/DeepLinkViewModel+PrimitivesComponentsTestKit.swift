// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSocialLink
import enum Gemstone.LinkType
@testable import PrimitivesComponents

extension DeepLinkViewModel {
    static func mock(_ linkType: LinkType = .telegram) -> DeepLinkViewModel? {
        let url = switch linkType {
        case .x: "https://x.com/GemWallet"
        case .discord: "https://discord.gg/aWkq5sj7SY"
        case .telegram: "https://t.me/gemwallet"
        case .gitHub: "https://github.com/gemwalletcom"
        case .youTube: "https://www.youtube.com/@gemwallet"
        case .reddit, .facebook, .website, .coingecko, .openSea, .instagram, .magicEden, .coinMarketCap, .tikTok: "https://example.com"
        }
        return DeepLinkViewModel(GemSocialLink(linkType: linkType, url: url, host: nil))
    }
}
