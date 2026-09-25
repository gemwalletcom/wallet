// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemListRowTitle
import struct Gemstone.GemListSection
import enum Gemstone.GemLockPeriod
import struct Gemstone.GemSecurityInput
import protocol Gemstone.GemSettingsServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents

@Observable
@MainActor
public final class SecurityViewModel {
    private let service: any BiometryAuthenticatable
    private let settings: any GemSettingsServiceProtocol
    private let preferences: ObservablePreferences

    static let reason: String = Localized.Settings.Security.authentication

    var isPresentingAlertMessage: AlertMessage?
    var isPresentingLockPeriods: Bool = false
    var isEnabled: Bool
    private var storedLockPeriod: GemLockPeriod
    var isPrivacyLockEnabled: Bool

    public init(
        service: any BiometryAuthenticatable,
        settings: any GemSettingsServiceProtocol,
        preferences: ObservablePreferences,
    ) {
        self.service = service
        self.settings = settings
        self.preferences = preferences

        storedLockPeriod = service.lockPeriod
        isEnabled = service.requiresAuthentication
        isPrivacyLockEnabled = service.isPrivacyLockEnabled
    }

    var title: String {
        Localized.Settings.security
    }

    var lockPeriodTitle: String {
        Localized.Lock.requireAuthentication
    }

    var allLockPeriods: [GemLockPeriod] {
        GemConstants.lockPeriods
    }

    private var authenticationName: String? {
        switch service.availableAuthentication {
        case .biometrics: KeystoreAuthentication.availableBiometryName
        case .passcode, .none: .none
        }
    }
}

public extension SecurityViewModel {
    var sections: [GemListSection] {
        settings.securitySections(
            input: GemSecurityInput(
                authenticationEnabled: isEnabled,
                authenticationName: authenticationName,
                lockPeriod: storedLockPeriod.title,
                privacyLockEnabled: isPrivacyLockEnabled,
                privacyLockSupported: true,
                hideBalanceEnabled: preferences.isHideBalanceEnabled,
            ),
        )
    }
}

// MARK: - Business Logic

extension SecurityViewModel {
    func onToggle(_ title: GemListRowTitle, _ isOn: Bool) {
        switch title {
        case .authentication:
            isEnabled = isOn
            Task { await toggleBiometrics() }
        case .privacyLock:
            isPrivacyLockEnabled = isOn
            togglePrivacyLock()
        case .hideBalance:
            preferences.isHideBalanceEnabled = isOn
        default:
            break
        }
    }

    func onSelect(_ title: GemListRowTitle) {
        switch title {
        case .lockPeriod: isPresentingLockPeriods = true
        default: break
        }
    }

    func toggleBiometrics() async {
        guard isEnabled != service.requiresAuthentication else { return }
        do {
            try await service.enableAuthentication(isEnabled, reason: SecurityViewModel.reason)
            isPrivacyLockEnabled = service.isPrivacyLockEnabled
            storedLockPeriod = service.lockPeriod
        } catch let error as BiometryAuthenticationError {
            if let text = error.promptOutcome.errorText() {
                isPresentingAlertMessage = AlertMessage(message: text.text)
            }
            isEnabled.toggle()
        } catch {
            isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
            isEnabled.toggle()
        }
    }

    func togglePrivacyLock() {
        guard isPrivacyLockEnabled != service.isPrivacyLockEnabled else { return }
        do {
            try service.togglePrivacyLock(enabled: isPrivacyLockEnabled)
        } catch {
            isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
            isPrivacyLockEnabled.toggle()
        }
    }

    func updateLockPeriod(to period: GemLockPeriod) {
        guard period != storedLockPeriod else { return }
        let previous = storedLockPeriod
        storedLockPeriod = period
        do {
            try service.update(period: period)
        } catch {
            isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
            storedLockPeriod = previous
        }
    }
}
