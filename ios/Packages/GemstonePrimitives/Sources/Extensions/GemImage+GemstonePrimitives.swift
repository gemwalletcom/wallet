// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemImage

public extension GemImage {
    var imageURL: URL? {
        URL(string: url())
    }
}
