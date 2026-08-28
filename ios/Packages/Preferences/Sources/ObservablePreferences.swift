// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesServiceProtocol
import GemstonePrimitives
import Primitives
import SwiftUI

@Observable
public final class ObservablePreferences: Sendable {
    public let preferences: Preferences
    private let preferencesService: any GemPreferencesServiceProtocol

    public init(preferences: Preferences, preferencesService: any GemPreferencesServiceProtocol) {
        self.preferences = preferences
        self.preferencesService = preferencesService
    }

    @ObservationIgnored
    public var isHideBalanceEnabled: Bool {
        get {
            access(keyPath: \.isHideBalanceEnabled)
            return preferencesService.isHideBalanceEnabled()
        }
        set {
            withMutation(keyPath: \.isHideBalanceEnabled) {
                write { try preferencesService.setHideBalanceEnabled(enabled: newValue) }
            }
        }
    }

    @ObservationIgnored
    public var isDeveloperEnabled: Bool {
        get {
            access(keyPath: \.isDeveloperEnabled)
            return preferencesService.isDeveloperEnabled()
        }
        set {
            withMutation(keyPath: \.isDeveloperEnabled) {
                write { try preferencesService.setDeveloperEnabled(enabled: newValue) }
            }
        }
    }

    @ObservationIgnored
    public var currentWalletId: String? {
        get {
            access(keyPath: \.currentWalletId)
            return preferences.currentWalletId
        }
        set {
            withMutation(keyPath: \.currentWalletId) {
                preferences.currentWalletId = newValue
            }
        }
    }

    @ObservationIgnored
    public var isAcceptTermsCompleted: Bool {
        access(keyPath: \.isAcceptTermsCompleted)
        return preferencesService.isAcceptTermsCompleted()
    }

    public func acceptTerms() {
        withMutation(keyPath: \.isAcceptTermsCompleted) {
            write { try preferencesService.setAcceptTermsCompleted() }
        }
    }

    @ObservationIgnored
    public var isPerpetualEnabled: Bool {
        get {
            access(keyPath: \.isPerpetualEnabled)
            return preferencesService.isPerpetualEnabled()
        }
        set {
            withMutation(keyPath: \.isPerpetualEnabled) {
                write { try preferencesService.setPerpetualEnabled(enabled: newValue) }
            }
        }
    }

    @ObservationIgnored
    public var appearance: Appearance {
        get {
            access(keyPath: \.appearance)
            return preferencesService.appearanceValue
        }
        set {
            withMutation(keyPath: \.appearance) {
                write { try preferencesService.setAppearanceValue(newValue) }
            }
        }
    }

    public func showPerpetuals(for wallet: Wallet) -> Bool {
        access(keyPath: \.isPerpetualEnabled)
        return (try? preferencesService.showPerpetuals(wallet: wallet.json())) ?? false
    }

    private func write(_ operation: () throws -> Void) {
        do {
            try operation()
        } catch {
            debugLog("preferences write error: \(error)")
        }
    }
}
