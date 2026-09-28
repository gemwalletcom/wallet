// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemWalletConnectError
import enum Gemstone.GemWalletConnectFailure
import Localization
import Primitives

extension GemWalletConnectFailure {
    var error: any Error {
        switch self {
        case .maliciousOrigin: GemWalletConnectError.InvalidOrigin
        case .expired: AnyError(Localized.WalletConnect.requestExpired)
        case let .failed(error): error
        }
    }
}
