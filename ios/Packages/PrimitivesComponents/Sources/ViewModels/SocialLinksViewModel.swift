// Copyright (c). Gem Wallet. All rights reserved.
import Foundation
import struct Gemstone.GemSocialLink

public struct SocialLinksViewModel {
    private let socialLinks: [GemSocialLink]

    public init(links: [GemSocialLink]) {
        socialLinks = links
    }

    var links: [InsightLink] {
        socialLinks.compactMap { AssetLinkViewModel($0).insightLink }
    }
}
