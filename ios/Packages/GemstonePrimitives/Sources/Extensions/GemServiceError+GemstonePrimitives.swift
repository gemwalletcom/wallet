// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemServiceError
import Localization

extension GemServiceError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .Api(msg), let .Gateway(msg), let .Store(msg), let .Core(msg), let .Platform(msg), let .InvalidInput(msg), let .NotFound(msg), let .Unsupported(msg): msg
        case .Cancelled: Localized.Errors.cancelled
        }
    }
}
