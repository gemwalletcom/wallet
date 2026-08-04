// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension Account: Identifiable {
    public var id: String {
        "\(chain)\(address)"
    }
}

public extension Account {
    var chainAddress: ChainAddress {
        ChainAddress(chain: chain, address: address)
    }
}
