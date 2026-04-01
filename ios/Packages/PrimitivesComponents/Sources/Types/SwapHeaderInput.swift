// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public struct SwapHeaderInput: Sendable {
    let from: AssetValuePrice
    let to: AssetValuePrice

    public init(from: AssetValuePrice, to: AssetValuePrice) {
        self.from = from
        self.to = to
    }
}
