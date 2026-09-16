// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAddNodeError
import enum Gemstone.GemAddNodeFailure
import Foundation
import Localization
import Primitives

extension GemAddNodeError {
    var failure: GemAddNodeFailure {
        switch self {
        case .InvalidUrl: .invalidUrl
        case .InvalidNetworkId: .invalidNetworkId
        case .Gateway: .unavailable
        }
    }
}

extension GemAddNodeFailure {
    var error: AnyError {
        switch self {
        case .invalidUrl: AnyError(Localized.Errors.invalidUrl)
        case .invalidNetworkId: AnyError(Localized.Errors.invalidNetworkId)
        case .unavailable: AnyError(Localized.Errors.errorOccurred)
        }
    }
}
