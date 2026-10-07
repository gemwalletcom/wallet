// Copyright (c). Gem Wallet. All rights reserved.

import LocalAuthentication
import Style

extension LABiometryType {
    static var available: LABiometryType {
        let context = LAContext()
        _ = context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: nil)
        return context.biometryType
    }

    var systemImage: String? {
        switch self {
        case .faceID: SystemImage.faceid
        case .touchID: SystemImage.touchid
        case .opticID: SystemImage.opticid
        case .none: .none
        @unknown default: .none
        }
    }

    var name: String? {
        switch self {
        case .faceID: "Face ID"
        case .touchID: "Touch ID"
        case .opticID: "Optic ID"
        case .none: .none
        @unknown default: .none
        }
    }
}
