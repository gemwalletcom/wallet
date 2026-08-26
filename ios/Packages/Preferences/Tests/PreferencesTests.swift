// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Preferences
import PreferencesTestKit
import Primitives
import Testing

struct PreferencesTests {
    @Test
    func removeLegacyPriceAlertsEnabledReturnsValueOnce() {
        let defaults = UserDefaults(suiteName: #function)!
        defaults.removePersistentDomain(forName: #function)
        let preferences = Preferences(defaults: defaults)

        #expect(preferences.removeLegacyPriceAlertsEnabled() == nil)

        defaults.set(true, forKey: Preferences.Keys.isPriceAlertsEnabled)
        #expect(preferences.removeLegacyPriceAlertsEnabled() == true)
        #expect(preferences.removeLegacyPriceAlertsEnabled() == nil)
    }

    private let preferences: Preferences = .mock()

    @Test
    func defaultPreferences() {
        #expect(preferences.currency == Currency.usd.rawValue)

        #expect(preferences.importFiatMappingsVersion == 0)
        #expect(preferences.importFiatPurchaseAssetsVersion == 0)
        #expect(preferences.localAssetsVersion == 0)
        #expect(preferences.fiatOnRampAssetsVersion == 0)
        #expect(preferences.fiatOffRampAssetsVersion == 0)
        #expect(preferences.swapAssetsVersion == 0)
        #expect(preferences.launchesCount == 0)
        #expect(preferences.subscriptionsVersion == 0)
        #expect(preferences.authenticationLockOption == 0)

        #expect(preferences.currentWalletId == nil)
        #expect(preferences.isSubscriptionsEnabled)
        #expect(!preferences.isPushNotificationsEnabled)
        #expect(!preferences.rateApplicationShown)
        #expect(!preferences.isDeveloperEnabled)
        #expect(!preferences.isHideBalanceEnabled)
        #expect(preferences.chartPeriod == .day)
        #expect(preferences.perpetualChartPeriod == .day)
        #expect(preferences.perpetualLeverage == 0)
        #expect(preferences.appearance == .system)
    }

    @Test
    func invalidateSubscriptionsBumpsVersion() {
        preferences.subscriptionsVersion = 8

        preferences.invalidateSubscriptions()

        #expect(preferences.subscriptionsVersion == 9)
    }

    @Test
    func testIncrementLaunchesCount() {
        #expect(preferences.launchesCount == 0)
        preferences.incrementLaunchesCount()
        #expect(preferences.launchesCount == 1)
    }


    @Test
    func updatePreferences() {
        preferences.currency = Currency.eur.rawValue
        #expect(preferences.currency == Currency.eur.rawValue)

        preferences.importFiatMappingsVersion = 1
        #expect(preferences.importFiatMappingsVersion == 1)

        preferences.importFiatPurchaseAssetsVersion = 2
        #expect(preferences.importFiatPurchaseAssetsVersion == 2)

        preferences.localAssetsVersion = 3
        #expect(preferences.localAssetsVersion == 3)

        preferences.fiatOnRampAssetsVersion = 4
        #expect(preferences.fiatOnRampAssetsVersion == 4)

        preferences.fiatOffRampAssetsVersion = 5
        #expect(preferences.fiatOffRampAssetsVersion == 5)

        preferences.swapAssetsVersion = 6
        #expect(preferences.swapAssetsVersion == 6)

        preferences.launchesCount = 7
        #expect(preferences.launchesCount == 7)

        preferences.subscriptionsVersion = 8
        #expect(preferences.subscriptionsVersion == 8)

        preferences.authenticationLockOption = 9
        #expect(preferences.authenticationLockOption == 9)

        preferences.currentWalletId = "wallet123"
        #expect(preferences.currentWalletId == "wallet123")

        preferences.isSubscriptionsEnabled = false
        #expect(!preferences.isSubscriptionsEnabled)

        preferences.isPushNotificationsEnabled = true
        #expect(preferences.isPushNotificationsEnabled)


        preferences.rateApplicationShown = true
        #expect(preferences.rateApplicationShown)

        preferences.isDeveloperEnabled = true
        #expect(preferences.isDeveloperEnabled)

        preferences.isHideBalanceEnabled = true
        #expect(preferences.isHideBalanceEnabled)

        preferences.setExplorerName(chain: .bitcoin, name: "btc")
        #expect(preferences.explorerName(chain: .bitcoin) == "btc")


        preferences.chartPeriod = .hour
        #expect(preferences.chartPeriod == .hour)

        preferences.perpetualChartPeriod = .month
        #expect(preferences.perpetualChartPeriod == .month)

        preferences.appearance = .dark
        #expect(preferences.appearance == .dark)

        preferences.perpetualLeverage = 25
        #expect(preferences.perpetualLeverage == 25)
    }

    @Test
    func testClear() {
        preferences.currency = Currency.eur.rawValue
        preferences.importFiatMappingsVersion = 1
        preferences.importFiatPurchaseAssetsVersion = 2
        preferences.localAssetsVersion = 3
        preferences.fiatOnRampAssetsVersion = 4
        preferences.fiatOffRampAssetsVersion = 5
        preferences.swapAssetsVersion = 6
        preferences.launchesCount = 7
        preferences.subscriptionsVersion = 8
        preferences.authenticationLockOption = 9
        preferences.currentWalletId = "wallet123"
        preferences.isSubscriptionsEnabled = false
        preferences.isPushNotificationsEnabled = true
        preferences.rateApplicationShown = true
        preferences.isDeveloperEnabled = true
        preferences.isHideBalanceEnabled = true
        preferences.setExplorerName(chain: .bitcoin, name: "btc")
        preferences.chartPeriod = .hour
        preferences.perpetualChartPeriod = .month
        preferences.perpetualLeverage = 25

        #expect(preferences.currency == Currency.eur.rawValue)
        #expect(preferences.importFiatMappingsVersion == 1)
        #expect(preferences.importFiatPurchaseAssetsVersion == 2)
        #expect(preferences.localAssetsVersion == 3)
        #expect(preferences.fiatOnRampAssetsVersion == 4)
        #expect(preferences.fiatOffRampAssetsVersion == 5)
        #expect(preferences.swapAssetsVersion == 6)
        #expect(preferences.launchesCount == 7)
        #expect(preferences.subscriptionsVersion == 8)
        #expect(preferences.authenticationLockOption == 9)
        #expect(preferences.currentWalletId == "wallet123")
        #expect(!preferences.isSubscriptionsEnabled)
        #expect(preferences.isPushNotificationsEnabled)
        #expect(preferences.rateApplicationShown)
        #expect(preferences.isDeveloperEnabled)
        #expect(preferences.isHideBalanceEnabled)
        #expect(preferences.explorerName(chain: .bitcoin) == "btc")
        #expect(preferences.chartPeriod == .hour)
        #expect(preferences.perpetualChartPeriod == .month)

        preferences.clear()

        #expect(preferences.currency == Currency.usd.rawValue)
        #expect(preferences.importFiatMappingsVersion == 0)
        #expect(preferences.importFiatPurchaseAssetsVersion == 0)
        #expect(preferences.localAssetsVersion == 0)
        #expect(preferences.fiatOnRampAssetsVersion == 0)
        #expect(preferences.fiatOffRampAssetsVersion == 0)
        #expect(preferences.swapAssetsVersion == 0)
        #expect(preferences.launchesCount == 0)
        #expect(preferences.subscriptionsVersion == 0)
        #expect(preferences.authenticationLockOption == 0)
        #expect(preferences.currentWalletId == nil)
        #expect(preferences.isSubscriptionsEnabled)
        #expect(!preferences.isPushNotificationsEnabled)
        #expect(!preferences.rateApplicationShown)
        #expect(!preferences.isDeveloperEnabled)
        #expect(!preferences.isHideBalanceEnabled)
        #expect(preferences.explorerName(chain: .bitcoin) == nil)
        #expect(preferences.chartPeriod == .day)
        #expect(preferences.perpetualChartPeriod == .day)
        #expect(preferences.perpetualLeverage == 0)
    }

    @Test
    func reinitializeReflectsExternalChanges() throws {
        let testDefaults = try #require(UserDefaults(suiteName: "testReinitialize"))
        testDefaults.removePersistentDomain(forName: "testReinitialize")

        let preferences = Preferences(defaults: testDefaults)

        #expect(preferences.currency == Currency.usd.rawValue)
        #expect(preferences.launchesCount == 0)
        #expect(preferences.currentWalletId == nil)

        testDefaults.set(Currency.eur.rawValue, forKey: "currency")
        testDefaults.set(5, forKey: "launches_count")
        testDefaults.set("walletXYZ", forKey: "currentWallet")

        #expect(preferences.currency == Currency.eur.rawValue)
        #expect(preferences.launchesCount == 5)
        #expect(preferences.currentWalletId == "walletXYZ")

        let newPrefs = Preferences(defaults: testDefaults)
        #expect(newPrefs.currency == Currency.eur.rawValue)
        #expect(newPrefs.launchesCount == 5)
        #expect(newPrefs.currentWalletId == "walletXYZ")

        testDefaults.removePersistentDomain(forName: "testReinitialize")
    }

    @Test
    func optionalNilAssignment() {
        preferences.currentWalletId = "wallet123"
        #expect(preferences.currentWalletId == "wallet123")

        preferences.currentWalletId = nil
        #expect(preferences.currentWalletId == nil)
    }
}
