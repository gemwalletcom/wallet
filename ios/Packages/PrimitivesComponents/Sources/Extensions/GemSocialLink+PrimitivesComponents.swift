// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemSocialLink
import Primitives

extension GemSocialLink {
    var listItem: ListItemModel {
        ListItemModel(title: linkType.title, subtitle: host, imageStyle: .settings(assetImage: .image(linkType.image)))
    }

    var deepLink: URL? {
        guard let path = url.asURL?.path().trimmingPrefix("/") else { return nil }

        return switch linkType {
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
