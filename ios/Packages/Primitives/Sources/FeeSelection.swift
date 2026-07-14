// Copyright (c). Gem Wallet. All rights reserved.

import BigInt

public enum FeeSelection: Equatable, Sendable {
    case preset(FeePriority)
    case custom(BigInt)

    public var presetPriority: FeePriority? {
        switch self {
        case let .preset(priority): priority
        case .custom: nil
        }
    }

    public var customValue: BigInt? {
        switch self {
        case .preset: nil
        case let .custom(value): value
        }
    }
}
