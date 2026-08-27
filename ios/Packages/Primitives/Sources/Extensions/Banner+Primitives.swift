// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension Banner: Identifiable {
    public var id: String {
        [wallet?.id.id, asset?.id.identifier, chain?.id, event.rawValue].compactMap(\.self).joined(separator: "_")
    }
}
