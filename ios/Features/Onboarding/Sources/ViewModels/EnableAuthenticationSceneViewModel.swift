// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.enableAuthenticationLabel
import GemstoneServices
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
        service.availableAuthentication.biometrySystemImage ?? SystemImage.lockFill
    }

    var enableTitle: String {
        enableAuthenticationLabel(authenticationName: authenticationName).text
    }

    var skipTitle: String {
        Localized.Common.skip
    }

    private var authenticationName: String? {
        service.availableAuthentication.biometryName
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
