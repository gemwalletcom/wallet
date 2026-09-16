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
        link.linkType.title
    }

    var image: Image {
        link.linkType.image
    }

    var url: URL? {
        link.url.asURL
    }

    var deepLink: URL? {
        DeepLinkViewModel(link).deepLink
    }
}
