// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemSocialLink
import Primitives

struct DeepLinkViewModel {
    let link: GemSocialLink

    init(_ link: GemSocialLink) {
        self.link = link
    }

    var deepLink: URL? {
        guard let path = link.url.asURL?.path().trimmingPrefix("/") else { return nil }

        return switch link.linkType {
        case .telegram: URL(string: "tg://resolve?domain=\(path)")
        case .x: URL(string: "twitter://user?screen_name=\(path)")
        case .youTube: URL(string: "youtube://www.youtube.com/\(path)")
        case .discord: URL(string: "https://discord.gg/\(path)")
        case .gitHub: URL(string: "https://github.com/\(path)")
        case .reddit, .facebook, .website, .coingecko, .openSea, .instagram, .magicEden, .coinMarketCap, .tikTok:
            nil
        }
    }
}
