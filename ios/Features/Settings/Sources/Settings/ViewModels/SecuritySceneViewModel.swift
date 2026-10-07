// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemListSection
import enum Gemstone.GemLockPeriod
import enum Gemstone.GemRowAction
import struct Gemstone.GemSecurityInput
import protocol Gemstone.GemSettingsServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents

@Observable
@MainActor
public final class SecuritySceneViewModel {
    private let service: any BiometryAuthenticatable
    private let settings: any GemSettingsServiceProtocol
    private let preferences: ObservablePreferences

    static let reason: String = Localized.Settings.Security.authentication

    var isPresentingAlertMessage: AlertMessage?
    var isPresentingLockPeriods: Bool = false
    var isEnabled: Bool
    private var storedLockPeriod: GemLockPeriod

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
        service.availableAuthentication.biometryName
    }
}

public extension SecuritySceneViewModel {
    var sections: [GemListSection] {
        preferences.changes
        return settings.securitySections(
            input: GemSecurityInput(
                authenticationEnabled: isEnabled,
                authenticationName: authenticationName,
                lockPeriod: storedLockPeriod.title,
            ),
        )
    }
}

// MARK: - Business Logic

extension SecuritySceneViewModel {
    func onToggle(_ action: GemRowAction, _ isOn: Bool) {
        switch action {
        case .authentication:
            isEnabled = isOn
            Task { await toggleBiometrics() }
        case .hideBalance:
            preferences.isHideBalanceEnabled = isOn
        default:
            break
        }
    }

    func onSelect(_ action: GemRowAction) {
        switch action {
        case .lockPeriod: isPresentingLockPeriods = true
        default: break
        }
    }

    func toggleBiometrics() async {
        guard isEnabled != service.requiresAuthentication else { return }
        do {
            try await service.enableAuthentication(isEnabled, reason: SecuritySceneViewModel.reason)
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
