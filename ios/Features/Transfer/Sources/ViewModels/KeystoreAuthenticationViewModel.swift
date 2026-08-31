// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices
import Style

struct KeystoreAuthenticationViewModel {
    let authentication: KeystoreAuthentication

    var authenticationImage: String? {
        switch authentication {
        case .biometrics: SystemImage.faceid
        case .passcode: SystemImage.lock
        case .none: .none
        }
    }
}
