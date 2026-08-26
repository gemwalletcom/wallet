// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public final class Preferences: @unchecked Sendable {
    public enum Constants {
        public static let appGroupIdentifier = "group.com.gemwallet.ios"
    }

    public enum Keys {
        static let currency = "currency"
        static let importFiatMappingsVersion = "migrate_fiat_mappings_version"
        static let importFiatPurchaseAssetsVersion = "migrate_fiat_purchase_assets_version"
        static let localAssetsVersion = "local_assets_version"
        static let swapSlippageBps = "swap_slippage_bps"
        static let launchesCount = "launches_count"
        static let subscriptionsVersion = "subscriptions_version"
        static let pushedDevice = "pushed_device"
        static let pushedSubscriptions = "pushed_subscriptions"
        static let currentWalletId = "currentWallet"
        static let isPushNotificationsEnabled = "is_push_notifications_enabled"
        static let isPriceAlertsEnabled = "is_price_alerts_enabled"
        static let isSubscriptionsEnabled = "is_subscriptions_enabled"
        static let rateApplicationShown = "rate_application_shown"
        static let authenticationLockOption = "authentication_lock_option"
        static let isDeveloperEnabled = "is_developer_enabled"
        static let isHideBalanceEnabled = "is_balance_privacy_enabled"
        static let isAcceptTermsCompleted = "is_accepted_terms"
        static let isWalletConnectActivated = "is_walletconnect_activated"
        static let chartPeriod = "chart_period"
        static let perpetualChartPeriod = "perpetual_chart_period"
        static let perpetualsMarketsUpdatedAt = "perpetual_markets_updated_at"
        static let perpetualPricesUpdatedAt = "perpetual_prices_updated_at"
        static let isPerpetualEnabled = "is_perpetual_enabled"
        static let perpetualLeverage = "perpetual_leverage"
        static let perpetualTakeProfit = "perpetual_take_profit"
        static let perpetualStopLoss = "perpetual_stop_loss"
        static let isDeviceRegistered = "is_device_registered"
        static let appearance = "appearance"
    }

    @ConfigurableDefaults(key: Keys.currency, defaultValue: Currency.usd.rawValue)
    public var currency: String

    @ConfigurableDefaults(key: Keys.importFiatMappingsVersion, defaultValue: 0)
    public var importFiatMappingsVersion: Int

    @ConfigurableDefaults(key: Keys.importFiatPurchaseAssetsVersion, defaultValue: 0)
    public var importFiatPurchaseAssetsVersion: Int

    @ConfigurableDefaults(key: Keys.localAssetsVersion, defaultValue: 0)
    public var localAssetsVersion: Int

    @ConfigurableDefaults(key: Keys.swapSlippageBps, defaultValue: 0)
    private var swapSlippageBpsRawValue: Int

    @ConfigurableDefaults(key: Keys.launchesCount, defaultValue: 0)
    public var launchesCount: Int

    @ConfigurableDefaults(key: Keys.subscriptionsVersion, defaultValue: 0)
    public var subscriptionsVersion: Int

    @ConfigurableDefaults(key: Keys.pushedDevice, defaultValue: .none)
    public var pushedDevice: String?

    @ConfigurableDefaults(key: Keys.pushedSubscriptions, defaultValue: .none)
    public var pushedSubscriptions: String?

    @ConfigurableDefaults(key: Keys.currentWalletId, defaultValue: .none)
    public var currentWalletId: String?

    @ConfigurableDefaults(key: Keys.isPushNotificationsEnabled, defaultValue: false)
    public var isPushNotificationsEnabled: Bool

    @ConfigurableDefaults(key: Keys.isSubscriptionsEnabled, defaultValue: true)
    public var isSubscriptionsEnabled: Bool

    @ConfigurableDefaults(key: Keys.rateApplicationShown, defaultValue: false)
    public var rateApplicationShown: Bool

    @ConfigurableDefaults(key: Keys.authenticationLockOption, defaultValue: 0)
    public var authenticationLockOption: Int

    @ConfigurableDefaults(key: Keys.isDeveloperEnabled, defaultValue: false)
    public var isDeveloperEnabled: Bool

    @ConfigurableDefaults(key: Keys.isHideBalanceEnabled, defaultValue: false)
    public var isHideBalanceEnabled: Bool

    @ConfigurableDefaults(key: Keys.isAcceptTermsCompleted, defaultValue: false)
    public var isAcceptTermsCompleted: Bool

    @ConfigurableDefaults(key: Keys.isWalletConnectActivated, defaultValue: nil)
    public var isWalletConnectActivated: Bool?

    @ConfigurableDefaults(key: Keys.chartPeriod, defaultValue: ChartPeriod.day.rawValue)
    private var chartPeriodRawValue: String

    @ConfigurableDefaults(key: Keys.perpetualChartPeriod, defaultValue: ChartPeriod.day.rawValue)
    private var perpetualChartPeriodRawValue: String

    @ConfigurableDefaults(key: Keys.perpetualsMarketsUpdatedAt, defaultValue: nil)
    public var perpetualMarketsUpdatedAt: Date?

    @ConfigurableDefaults(key: Keys.perpetualPricesUpdatedAt, defaultValue: nil)
    public var perpetualPricesUpdatedAt: Date?

    @ConfigurableDefaults(key: Keys.isPerpetualEnabled, defaultValue: false)
    public var isPerpetualEnabled: Bool

    @ConfigurableDefaults(key: Keys.perpetualLeverage, defaultValue: 0)
    public var perpetualLeverage: UInt8

    @ConfigurableDefaults(key: Keys.perpetualTakeProfit, defaultValue: 0)
    public var perpetualTakeProfit: UInt8

    @ConfigurableDefaults(key: Keys.perpetualStopLoss, defaultValue: 0)
    public var perpetualStopLoss: UInt8

    @ConfigurableDefaults(key: Keys.isDeviceRegistered, defaultValue: false)
    public var isDeviceRegistered: Bool

    @ConfigurableDefaults(key: Keys.appearance, defaultValue: Appearance.system.rawValue)
    private var appearanceRawValue: String

    public static let standard = Preferences()
    private let defaults: UserDefaults

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        configureAllProperties(with: defaults)
    }

    private func configureAllProperties(with defaults: UserDefaults) {
        let sharedDefaults = UserDefaults(suiteName: Constants.appGroupIdentifier)

        func configure<T>(_ keyPath: ReferenceWritableKeyPath<Preferences, ConfigurableDefaults<T>>, key: String, defaultValue: T, sharedDefaults: UserDefaults? = nil) {
            self[keyPath: keyPath] = ConfigurableDefaults(key: key, defaultValue: defaultValue, defaults: defaults, sharedDefaults: sharedDefaults)
        }
        configure(\._currency, key: Keys.currency, defaultValue: Currency.usd.rawValue, sharedDefaults: sharedDefaults)
        configure(\._importFiatMappingsVersion, key: Keys.importFiatMappingsVersion, defaultValue: 0)
        configure(\._importFiatPurchaseAssetsVersion, key: Keys.importFiatPurchaseAssetsVersion, defaultValue: 0)
        configure(\._localAssetsVersion, key: Keys.localAssetsVersion, defaultValue: 0)
        configure(\._swapSlippageBpsRawValue, key: Keys.swapSlippageBps, defaultValue: 0)
        configure(\._launchesCount, key: Keys.launchesCount, defaultValue: 0)
        configure(\._subscriptionsVersion, key: Keys.subscriptionsVersion, defaultValue: 0)
        configure(\._pushedDevice, key: Keys.pushedDevice, defaultValue: nil)
        configure(\._pushedSubscriptions, key: Keys.pushedSubscriptions, defaultValue: nil)
        configure(\._currentWalletId, key: Keys.currentWalletId, defaultValue: nil)
        configure(\._isPushNotificationsEnabled, key: Keys.isPushNotificationsEnabled, defaultValue: false)
        configure(\._isSubscriptionsEnabled, key: Keys.isSubscriptionsEnabled, defaultValue: true)
        configure(\._rateApplicationShown, key: Keys.rateApplicationShown, defaultValue: false)
        configure(\._authenticationLockOption, key: Keys.authenticationLockOption, defaultValue: 0)
        configure(\._isDeveloperEnabled, key: Keys.isDeveloperEnabled, defaultValue: false)
        configure(\._isHideBalanceEnabled, key: Keys.isHideBalanceEnabled, defaultValue: false)
        configure(\._isAcceptTermsCompleted, key: Keys.isAcceptTermsCompleted, defaultValue: false)
        configure(\._isWalletConnectActivated, key: Keys.isWalletConnectActivated, defaultValue: nil)
        configure(\._chartPeriodRawValue, key: Keys.chartPeriod, defaultValue: ChartPeriod.day.rawValue)
        configure(\._perpetualChartPeriodRawValue, key: Keys.perpetualChartPeriod, defaultValue: ChartPeriod.day.rawValue)
        configure(\._perpetualMarketsUpdatedAt, key: Keys.perpetualsMarketsUpdatedAt, defaultValue: nil)
        configure(\._perpetualPricesUpdatedAt, key: Keys.perpetualPricesUpdatedAt, defaultValue: nil)
        configure(\._isPerpetualEnabled, key: Keys.isPerpetualEnabled, defaultValue: false)
        configure(\._perpetualLeverage, key: Keys.perpetualLeverage, defaultValue: 0)
        configure(\._perpetualTakeProfit, key: Keys.perpetualTakeProfit, defaultValue: 0)
        configure(\._perpetualStopLoss, key: Keys.perpetualStopLoss, defaultValue: 0)
        configure(\._isDeviceRegistered, key: Keys.isDeviceRegistered, defaultValue: false)
        configure(\._appearanceRawValue, key: Keys.appearance, defaultValue: Appearance.system.rawValue)
    }

    public func removeLegacyPriceAlertsEnabled() -> Bool? {
        let enabled = defaults.object(forKey: Keys.isPriceAlertsEnabled) as? Bool
        defaults.removeObject(forKey: Keys.isPriceAlertsEnabled)
        return enabled
    }

    public func incrementLaunchesCount() {
        launchesCount += 1
    }

    public var hasCurrency: Bool {
        defaults.object(forKey: Keys.currency) != nil
    }

    public func clear() {
        for key in defaults.dictionaryRepresentation().keys {
            defaults.removeObject(forKey: key)
        }
    }


    public func invalidateSubscriptions() {
        subscriptionsVersion += 1
    }

    public var swapSlippage: SwapSlippage {
        get { swapSlippageBpsRawValue > 0 ? .manual(bps: UInt32(swapSlippageBpsRawValue)) : .auto }
        set {
            swapSlippageBpsRawValue = switch newValue {
            case .auto: 0
            case let .manual(bps): Int(bps)
            }
        }
    }

    public var chartPeriod: ChartPeriod {
        get { ChartPeriod(rawValue: chartPeriodRawValue) ?? .day }
        set { chartPeriodRawValue = newValue.rawValue }
    }

    public var perpetualChartPeriod: ChartPeriod {
        get { ChartPeriod(rawValue: perpetualChartPeriodRawValue) ?? .day }
        set { perpetualChartPeriodRawValue = newValue.rawValue }
    }

    public var appearance: Appearance {
        get { Appearance(rawValue: appearanceRawValue) ?? .system }
        set { appearanceRawValue = newValue.rawValue }
    }

    public func showPerpetuals(for wallet: Wallet) -> Bool {
        isPerpetualEnabled && wallet.hasPerpetualsSupport
    }
}
