// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Components
import enum Gemstone.GemSecurityRow
import protocol Gemstone.GemSettingsServiceProtocol
import Foundation
import GemstoneServices
import Localization

@Observable
@MainActor
public final class SecurityViewModel {
    private let service: any BiometryAuthenticatable
    private let settings: any GemSettingsServiceProtocol
    private let preferences: ObservablePreferences

    static let reason: String = Localized.Settings.Security.authentication

    var isPresentingAlertMessage: AlertMessage?
    var isEnabled: Bool
    private var storedLockPeriod: LockPeriod
    var isPrivacyLockEnabled: Bool
    var isHideBalanceEnabled: Bool {
        get {
            preferences.isHideBalanceEnabled
        }
        set {
            preferences.isHideBalanceEnabled = newValue
        }
    }

    var lockPeriod: LockPeriod {
        get { storedLockPeriod }
        set { updateLockPeriod(to: newValue) }
    }

    var allLockPeriods: [LockPeriod] {
        LockPeriod.offered
    }

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

    var sections: [ListSection<SecurityRow>] {
        settings.securitySections(authenticationEnabled: isEnabled).enumerated().map { index, section in
            ListSection(id: "\(index)", title: nil, image: nil, values: section.rows.map(securityRow))
        }
    }

    private func securityRow(_ row: GemSecurityRow) -> SecurityRow {
        switch row {
        case .authentication: .authentication
        case .lockPeriod: .lockPeriod
        case .privacyLock: .privacyLock
        case .hideBalance: .hideBalance
        }
    }

    var title: String {
        Localized.Settings.security
    }

    var errorTitle: String {
        Localized.Errors.errorOccurred
    }

    var privacyLockTitle: String {
        Localized.Lock.privacyLock
    }

    var hideBalanceTitle: String {
        Localized.Settings.hideBalance
    }

    var lockPeriodTitle: String {
        Localized.Lock.requireAuthentication
    }

    var authenticationFooter: String {
        Localized.Lock.footer
    }

    var authenticationTitle: String {
        service.availableAuthentication.enableTitle
    }
}

// MARK: - Business Logic

extension SecurityViewModel {
    func toggleBiometrics() async {
        guard isEnabled != service.requiresAuthentication else { return }
        do {
            try await service.enableAuthentication(isEnabled, reason: SecurityViewModel.reason)
            isPrivacyLockEnabled = service.isPrivacyLockEnabled
            storedLockPeriod = service.lockPeriod
        } catch let error as BiometryAuthenticationError {
            if !error.isAuthenticationCancelled {
                isPresentingAlertMessage = AlertMessage(message: error.localizedDescription)
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

    func updateLockPeriod(to period: LockPeriod) {
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
