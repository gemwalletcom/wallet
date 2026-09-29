// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension Node: Identifiable {
    public var id: String {
        "\(url)"
    }
}

extension Node: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(url)
    }

    public static func == (lhs: Node, rhs: Node) -> Bool {
        lhs.url == rhs.url
    }
}
