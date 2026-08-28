// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Preferences
import PreferencesTestKit
import Primitives
import Testing

struct PreferencesTests {
    private let preferences: Preferences = .mock()

    @Test
    func defaultPreferences() {
        #expect(preferences.currency == Currency.usd.rawValue)

        #expect(preferences.importFiatMappingsVersion == 0)
        #expect(preferences.importFiatPurchaseAssetsVersion == 0)
        #expect(preferences.launchesCount == 0)
        #expect(preferences.authenticationLockOption == 0)

        #expect(preferences.currentWalletId == nil)
        #expect(preferences.isSubscriptionsEnabled)
        #expect(!preferences.isPushNotificationsEnabled)
        #expect(!preferences.rateApplicationShown)
        #expect(preferences.chartPeriod == .day)
        #expect(preferences.perpetualChartPeriod == .day)
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

        preferences.launchesCount = 7
        #expect(preferences.launchesCount == 7)


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



        preferences.chartPeriod = .hour
        #expect(preferences.chartPeriod == .hour)

        preferences.perpetualChartPeriod = .month
        #expect(preferences.perpetualChartPeriod == .month)


    }

    @Test
    func testClear() {
        preferences.currency = Currency.eur.rawValue
        preferences.importFiatMappingsVersion = 1
        preferences.importFiatPurchaseAssetsVersion = 2
        preferences.launchesCount = 7
        preferences.authenticationLockOption = 9
        preferences.currentWalletId = "wallet123"
        preferences.isSubscriptionsEnabled = false
        preferences.isPushNotificationsEnabled = true
        preferences.rateApplicationShown = true
        preferences.chartPeriod = .hour
        preferences.perpetualChartPeriod = .month

        #expect(preferences.currency == Currency.eur.rawValue)
        #expect(preferences.importFiatMappingsVersion == 1)
        #expect(preferences.importFiatPurchaseAssetsVersion == 2)
        #expect(preferences.launchesCount == 7)
        #expect(preferences.authenticationLockOption == 9)
        #expect(preferences.currentWalletId == "wallet123")
        #expect(!preferences.isSubscriptionsEnabled)
        #expect(preferences.isPushNotificationsEnabled)
        #expect(preferences.rateApplicationShown)
        #expect(preferences.chartPeriod == .hour)
        #expect(preferences.perpetualChartPeriod == .month)

        preferences.clear()

        #expect(preferences.currency == Currency.usd.rawValue)
        #expect(preferences.importFiatMappingsVersion == 0)
        #expect(preferences.importFiatPurchaseAssetsVersion == 0)
        #expect(preferences.launchesCount == 0)
        #expect(preferences.authenticationLockOption == 0)
        #expect(preferences.currentWalletId == nil)
        #expect(preferences.isSubscriptionsEnabled)
        #expect(!preferences.isPushNotificationsEnabled)
        #expect(!preferences.rateApplicationShown)
        #expect(preferences.chartPeriod == .day)
        #expect(preferences.perpetualChartPeriod == .day)
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
