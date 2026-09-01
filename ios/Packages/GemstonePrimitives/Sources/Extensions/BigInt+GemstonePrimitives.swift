// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import typealias Gemstone.GemBigInt
import Foundation

public extension BigInt {
    init(core value: GemBigInt) {
        guard let parsed = BigInt(value, radix: 10) else {
            preconditionFailure("failed to decode BigInt from Core: \(value)")
        }
        self = parsed
    }
}
