// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension AssetLink {
    var linkType: LinkType? {
        LinkType(rawValue: name)
    }
}

public extension [AssetLink] {
    var sortedByType: [AssetLink] {
        sorted { ($0.linkType?.order ?? 0) > ($1.linkType?.order ?? 0) }
    }
}
