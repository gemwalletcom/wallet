// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension Banner: Identifiable {
    public var id: String {
        [wallet?.id, asset?.id.identifier, chain?.id, event.rawValue].compactMap(\.self).joined(separator: "_")
    }
}

extension Banner: Comparable {
    public static func < (lhs: Banner, rhs: Banner) -> Bool {
        (lhs.state, lhs.event) < (rhs.state, rhs.event)
    }
}
