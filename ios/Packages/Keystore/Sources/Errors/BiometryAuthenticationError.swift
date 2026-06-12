// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import LocalAuthentication

public enum BiometryAuthenticationError: Error, Equatable {
    case biometryUnavailable
    case cancelledByUser
    case cancelledBySystem
    case authenticationFailed

    init(error: NSError) {
        switch error {
        case let laError as LAError:
            switch laError.code {
            case .biometryNotAvailable,
                 .passcodeNotSet:
                self = .biometryUnavailable
            case .userCancel,
                 .userFallback,
                 .biometryLockout:
                self = .cancelledByUser
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

    public var isAuthenticationCancelled: Bool {
        switch self {
        case .cancelledByUser, .cancelledBySystem: true
        case .biometryUnavailable, .authenticationFailed: false
        }
    }
}
