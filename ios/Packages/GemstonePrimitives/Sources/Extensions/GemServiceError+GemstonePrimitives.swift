// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemServiceError

extension GemServiceError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .Api(msg), let .Gateway(msg), let .Store(msg), let .Status(msg), let .Core(msg), let .Platform(msg): msg
        case let .UnknownCurrency(currency): "Unknown currency: \(currency)"
        case .Cancelled: "Cancelled"
        }
    }
}
