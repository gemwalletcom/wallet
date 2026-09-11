// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemSocialLink
import Localization
import Primitives
import Style
import SwiftUI

struct AssetLinkViewModel {
    let link: GemSocialLink

    init(_ link: GemSocialLink) {
        self.link = link
    }

    var insightLink: InsightLink? {
        guard let url else {
            return .none
        }
        return InsightLink(
            title: name,
            subtitle: link.host,
            url: url,
            deepLink: deepLink,
            image: AssetImage.image(image),
        )
    }

    var name: String {
        switch link.linkType {
        case .x: Localized.Social.x
        case .discord: Localized.Social.discord
        case .reddit: Localized.Social.reddit
        case .telegram: Localized.Social.telegram
        case .gitHub: Localized.Social.github
        case .youTube: Localized.Social.youtube
        case .facebook: Localized.Social.facebook
        case .website: Localized.Social.website
        case .coingecko: Localized.Social.coingecko
        case .coinMarketCap: Localized.Social.coinmarketcap
        case .openSea: Localized.Social.opensea
        case .instagram: Localized.Social.instagram
        case .magicEden: Localized.Social.magiceden
        case .tikTok: Localized.Social.tiktok
        }
    }

    var image: Image {
        switch link.linkType {
        case .x: Images.Social.x
        case .discord: Images.Social.discord
        case .reddit: Images.Social.reddit
        case .telegram: Images.Social.telegram
        case .gitHub: Images.Social.github
        case .youTube: Images.Social.youtube
        case .facebook: Images.Social.facebook
        case .website: Images.Social.website
        case .coingecko: Images.Social.coingecko
        case .coinMarketCap: Images.Social.coinmarketcap
        case .openSea: Images.Social.opensea
        case .instagram: Images.Social.instagram
        case .magicEden: Images.Social.magiceden
        case .tikTok: Images.Social.tiktok
        }
    }

    var url: URL? {
        link.url.asURL
    }

    var deepLink: URL? {
        DeepLinkViewModel(link).deepLink
    }
}
