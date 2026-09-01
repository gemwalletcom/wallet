// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemKeystoreAuthentication
import GemstoneServices
import Style

struct KeystoreAuthenticationViewModel {
    let authentication: GemKeystoreAuthentication

    var authenticationImage: String? {
        switch authentication {
        case .biometrics: SystemImage.faceid
        case .passcode: SystemImage.lock
        case .none: .none
        }
    }
}
