// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemKeystoreAuthentication
import Keychain
import LocalAuthentication

extension GemKeystoreAuthentication {
    var policy: AuthenticationPolicy {
        switch self {
        case .biometrics:
            [.biometryAny, .or, .devicePasscode]
        case .passcode:
            [.devicePasscode]
        case .none:
            []
        }
    }

    static var availableAuthenticationType: GemKeystoreAuthentication {
        let context = LAContext()
        do {
            try context.canEvaluatePolicyThrowing(policy: .deviceOwnerAuthenticationWithBiometrics)
            return .biometrics
        } catch {
            do {
                try context.canEvaluatePolicyThrowing(policy: .deviceOwnerAuthentication)
                return .passcode
            } catch {
                return .none
            }
        }
    }
}

private extension LAContext {
    func canEvaluatePolicyThrowing(policy: LAPolicy) throws {
        var error: NSError?
        canEvaluatePolicy(policy, error: &error)
        if let error {
            throw error
        }
    }
}
