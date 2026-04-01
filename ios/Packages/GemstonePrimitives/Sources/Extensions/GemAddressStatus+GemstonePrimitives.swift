// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.AddressStatus
import Primitives

public extension Gemstone.AddressStatus {
    func map() -> Primitives.AddressStatus {
        switch self {
        case .multiSignature: .multiSignature
        }
    }
}
