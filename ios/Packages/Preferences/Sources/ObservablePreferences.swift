// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SwiftUI

@Observable
public final class ObservablePreferences: Sendable {
    public static let `default` = ObservablePreferences()

    public let preferences: Preferences

    public init(preferences: Preferences = .standard) {
        self.preferences = preferences
    }

    public func invalidateSubscriptions() {
        preferences.invalidateSubscriptions()
    }

    @ObservationIgnored
    public var isHideBalanceEnabled: Bool {
        get {
            access(keyPath: \.isHideBalanceEnabled)
            return preferences.isHideBalanceEnabled
        }
        set {
            withMutation(keyPath: \.isHideBalanceEnabled) {
                preferences.isHideBalanceEnabled = newValue
            }
        }
    }

    @ObservationIgnored
    public var isPriceAlertsEnabled: Bool {
        get {
            access(keyPath: \.isPriceAlertsEnabled)
            return preferences.isPriceAlertsEnabled
        }
        set {
            withMutation(keyPath: \.isPriceAlertsEnabled) {
                preferences.isPriceAlertsEnabled = newValue
            }
        }
    }

    @ObservationIgnored
    public var isDeveloperEnabled: Bool {
        get {
            access(keyPath: \.isDeveloperEnabled)
            return preferences.isDeveloperEnabled
        }
        set {
            withMutation(keyPath: \.isDeveloperEnabled) {
                preferences.isDeveloperEnabled = newValue
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
        get {
            access(keyPath: \.isAcceptTermsCompleted)
            return preferences.isAcceptTermsCompleted
        }
        set {
            withMutation(keyPath: \.isAcceptTermsCompleted) {
                preferences.isAcceptTermsCompleted = newValue
            }
        }
    }

    @ObservationIgnored
    public var isPerpetualEnabled: Bool {
        get {
            access(keyPath: \.isPerpetualEnabled)
            return preferences.isPerpetualEnabled
        }
        set {
            withMutation(keyPath: \.isPerpetualEnabled) {
                preferences.isPerpetualEnabled = newValue
            }
        }
    }

    @ObservationIgnored
    public var perpetualLeverage: UInt8 {
        get {
            access(keyPath: \.perpetualLeverage)
            return preferences.perpetualLeverage
        }
        set {
            withMutation(keyPath: \.perpetualLeverage) {
                preferences.perpetualLeverage = newValue
            }
        }
    }

    @ObservationIgnored
    public var perpetualTakeProfit: UInt8 {
        get {
            access(keyPath: \.perpetualTakeProfit)
            return preferences.perpetualTakeProfit
        }
        set {
            withMutation(keyPath: \.perpetualTakeProfit) {
                preferences.perpetualTakeProfit = newValue
            }
        }
    }

    @ObservationIgnored
    public var perpetualStopLoss: UInt8 {
        get {
            access(keyPath: \.perpetualStopLoss)
            return preferences.perpetualStopLoss
        }
        set {
            withMutation(keyPath: \.perpetualStopLoss) {
                preferences.perpetualStopLoss = newValue
            }
        }
    }

    public func showPerpetuals(for wallet: Wallet) -> Bool {
        access(keyPath: \.isPerpetualEnabled)
        return preferences.showPerpetuals(for: wallet)
    }
}

// MARK: - EnvironmentValues

public extension EnvironmentValues {
    @Entry var observablePreferences: ObservablePreferences = .default
}
