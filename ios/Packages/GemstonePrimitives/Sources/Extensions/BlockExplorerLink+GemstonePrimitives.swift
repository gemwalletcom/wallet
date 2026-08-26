// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemBlockExplorerLink
import Primitives

public extension Primitives.BlockExplorerLink {
    init(_ link: Gemstone.GemBlockExplorerLink) {
        self.init(name: link.name, link: link.link)
    }
}
