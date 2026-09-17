// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletConnectError
import enum Gemstone.GemWalletConnectFailure
import Foundation
import Localization
import Primitives

extension GemWalletConnectFailure {
    var error: any Error {
        switch self {
        case .maliciousOrigin: GemWalletConnectError.InvalidOrigin
        case .expired: AnyError(Localized.WalletConnect.requestExpired)
        case let .failed(message): AnyError(message)
        }
    }
}
