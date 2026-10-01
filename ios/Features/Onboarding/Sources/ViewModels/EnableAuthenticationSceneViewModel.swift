// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.enableAuthenticationLabel
import GemstoneServices
import LocalAuthentication
import Localization
import Primitives
import PrimitivesComponents
import Style

@Observable
@MainActor
final class EnableAuthenticationSceneViewModel {
    private let service: any BiometryAuthenticatable
    private let preferences: ObservablePreferences
    private let onComplete: VoidAction

    var isPresentingAlertMessage: AlertMessage?

    init(
        service: any BiometryAuthenticatable,
        preferences: ObservablePreferences,
        onComplete: VoidAction,
    ) {
        self.service = service
        self.preferences = preferences
        self.onComplete = onComplete
    }

    var title: String {
        authenticationName ?? Localized.Lock.passcode
    }

    var description: String {
        Localized.Lock.footer
    }

    var image: String {
        switch service.availableAuthentication {
        case .biometrics:
            switch KeystoreAuthentication.availableBiometryType {
            case .faceID: SystemImage.faceid
            case .touchID: SystemImage.touchid
            case .opticID: SystemImage.opticid
            case .none: SystemImage.lockFill
            @unknown default: SystemImage.lockFill
            }
        case .passcode, .none: SystemImage.lockFill
        }
    }

    var enableTitle: String {
        enableAuthenticationLabel(authenticationName: authenticationName).text
    }

    var skipTitle: String {
        Localized.Common.skip
    }

    private var authenticationName: String? {
        switch service.availableAuthentication {
        case .biometrics: KeystoreAuthentication.availableBiometryName
        case .passcode, .none: .none
        }
    }
}

// MARK: - Actions

extension EnableAuthenticationSceneViewModel {
    func enable() async {
        do {
            try await service.enableAuthentication(true, reason: Localized.Settings.Security.authentication)
            complete()
        } catch let error as BiometryAuthenticationError {
            if let text = error.promptOutcome.errorText() {
                isPresentingAlertMessage = AlertMessage(message: text.text)
            }
        } catch {
            isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
        }
    }

    func skip() {
        complete()
    }

    private func complete() {
        preferences.setAuthenticationOffered()
        onComplete?()
    }
}
