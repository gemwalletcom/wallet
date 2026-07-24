// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public enum TransferAmountValue: Sendable, Equatable, Hashable {
    case exact(BigInt)
    case max(BigInt)

    public var value: BigInt {
        switch self {
        case let .exact(value), let .max(value): value
        }
    }
}
