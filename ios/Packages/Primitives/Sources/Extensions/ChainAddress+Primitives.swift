// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension ChainAddress: Identifiable {
    public var id: String {
        "\(chain.rawValue)_\(address)"
    }
}
