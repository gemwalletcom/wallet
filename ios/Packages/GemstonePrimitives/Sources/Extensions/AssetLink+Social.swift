// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension AssetLink {
    var linkType: LinkType? {
        LinkType(rawValue: name)
    }
}
