// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

extension KeystoreError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .missingPassword: Localized.Errors.keystoreAccess
        }
    }
}
