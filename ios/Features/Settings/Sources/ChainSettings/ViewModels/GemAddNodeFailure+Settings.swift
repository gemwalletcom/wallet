// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAddNodeError
import enum Gemstone.GemAddNodeFailure
import Foundation

extension GemAddNodeError {
    var failure: GemAddNodeFailure {
        switch self {
        case .InvalidUrl: .invalidUrl
        case .InvalidNetworkId: .invalidNetworkId
        case .Gateway: .unavailable
        }
    }
}
