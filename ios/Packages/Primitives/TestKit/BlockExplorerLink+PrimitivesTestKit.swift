// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension BlockExplorerLink {
    static func mock() -> BlockExplorerLink {
        BlockExplorerLink(name: "Mock", link: "https://mock.com")
    }
}
