// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.WalletConnectionVerificationStatus
import enum Primitives.WalletConnectionVerificationStatus

public extension Gemstone.WalletConnectionVerificationStatus {
    func map() -> Primitives.WalletConnectionVerificationStatus {
        switch self {
        case .verified: .verified
        case .unknown: .unknown
        case .invalid: .invalid
        case .malicious: .malicious
        }
    }
}
