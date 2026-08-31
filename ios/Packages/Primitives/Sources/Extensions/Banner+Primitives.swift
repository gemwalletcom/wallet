// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension Banner: Identifiable {
    public var id: String {
        [walletId?.id, asset?.id.identifier, event.rawValue].compactMap(\.self).joined(separator: "_")
    }
}
