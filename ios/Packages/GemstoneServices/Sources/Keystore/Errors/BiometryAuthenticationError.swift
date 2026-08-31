// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemSecurityService
import enum Gemstone.GemAuthPromptOutcome
import LocalAuthentication

public enum BiometryAuthenticationError: Error, Equatable {
    case biometryUnavailable
    case cancelledByUser
    case cancelledBySystem
    case lockedOut
    case authenticationFailed

    init(error: NSError) {
        switch error {
        case let laError as LAError:
            switch laError.code {
            case .biometryNotAvailable,
                 .passcodeNotSet:
                self = .biometryUnavailable
            case .userCancel,
                 .userFallback:
                self = .cancelledByUser
            case .biometryLockout:
                self = .lockedOut
            case .systemCancel,
                 .appCancel:
                self = .cancelledBySystem
            default:
                self = .authenticationFailed
            }
        default:
            self = .authenticationFailed
        }
    }

    public var promptOutcome: GemAuthPromptOutcome {
        switch self {
        case .biometryUnavailable: .unavailable
        case .cancelledByUser: .cancelledByUser
        case .cancelledBySystem: .cancelledBySystem
        case .lockedOut: .lockedOut
        case .authenticationFailed: .failed
        }
    }

    public var isAuthenticationCancelled: Bool {
        GemSecurityService().isAuthCancelled(outcome: promptOutcome)
    }
}
