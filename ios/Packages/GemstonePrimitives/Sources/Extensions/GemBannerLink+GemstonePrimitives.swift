// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemBannerLink

public extension GemBannerLink {
    var url: URL? {
        switch self {
        case let .docs(item): AppUrl.docs(item)
        case let .external(url): URL(string: url)
        }
    }
}
