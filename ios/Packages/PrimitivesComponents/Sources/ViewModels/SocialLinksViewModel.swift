// Copyright (c). Gem Wallet. All rights reserved.
import Foundation
import struct Gemstone.GemSocialLinks
import GemstonePrimitives
import Primitives

public struct SocialLinksViewModel {
    public let assetLinks: [AssetLink]

    public init(assetLinks: [AssetLink]) {
        self.assetLinks = assetLinks
    }

    var links: [InsightLink] {
        GemSocialLinks(links: assetLinks.map { $0.map() })
            .sorted()
            .compactMap { AssetLinkViewModel($0.map()).insightLink }
    }
}
