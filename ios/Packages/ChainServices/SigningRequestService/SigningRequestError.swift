// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization

public enum SigningRequestError: LocalizedError {
    case userCancelled

    public var errorDescription: String? {
        switch self {
        case .userCancelled: Localized.Errors.Connections.userCancelled
        }
    }
}
