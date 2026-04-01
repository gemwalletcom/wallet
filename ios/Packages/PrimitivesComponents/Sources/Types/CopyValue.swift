// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives

public enum CopyValue: Sendable, Equatable, Hashable {
    case plain(String)
    case address(value: String, chain: Chain?)

    public var rawValue: String {
        switch self {
        case let .plain(value): value
        case let .address(value, _): value
        }
    }

    public var displayValue: String {
        switch self {
        case let .plain(value): value
        case let .address(value, chain):
            AddressFormatter(style: .short, address: value, chain: chain).value()
        }
    }
}
