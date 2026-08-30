// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesServiceProtocol
import GemstonePrimitives
import Primitives
import SwiftUI

@Observable
public final class ObservablePreferences: Sendable {
    private let preferencesService: any GemPreferencesServiceProtocol

    public init(preferencesService: any GemPreferencesServiceProtocol) {
        self.preferencesService = preferencesService
    }

    @ObservationIgnored
    public var currency: String {
        get {
            access(keyPath: \.currency)
            return preferencesService.currencyCode
        }
        set {
            withMutation(keyPath: \.currency) {
                guard let currency = Currency(rawValue: newValue) else { return }
                write { try preferencesService.setCurrencyValue(currency) }
            }
        }
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
    public var isAcceptTermsCompleted: Bool {
        access(keyPath: \.isAcceptTermsCompleted)
        return preferencesService.isAcceptTermsCompleted()
    }

    public func reload() {
        withMutation(keyPath: \.currency) {}
        withMutation(keyPath: \.isHideBalanceEnabled) {}
        withMutation(keyPath: \.isDeveloperEnabled) {}
        withMutation(keyPath: \.isAcceptTermsCompleted) {}
        withMutation(keyPath: \.isPerpetualEnabled) {}
        withMutation(keyPath: \.appearance) {}
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
