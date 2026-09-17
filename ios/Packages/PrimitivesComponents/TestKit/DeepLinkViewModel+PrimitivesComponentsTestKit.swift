// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.communityLinks
import enum Gemstone.LinkType
@testable import PrimitivesComponents

extension DeepLinkViewModel {
    static func mock(_ linkType: LinkType = .telegram) -> DeepLinkViewModel? {
        communityLinks().first { $0.linkType == linkType }.map { DeepLinkViewModel($0) }
    }
}
