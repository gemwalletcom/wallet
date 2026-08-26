// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemWalletSource
import Primitives

public extension WalletSource {
    func map() -> GemWalletSource {
        switch self {
        case .create: .create
        case .import: .import
        }
    }
}
