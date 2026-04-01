// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public extension Balance {
    static let zero: Balance = .init(available: BigInt.zero)

    var total: BigInt {
        available + frozen + locked + staked + pending + rewards + earn
    }
}
