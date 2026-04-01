// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Preferences
import PreferencesTestKit
import Primitives
import Testing

struct SharedPreferencesTests {
    @Test
    func readWriteCurrency() {
        let mockDefaults = UserDefaults.mock()
        var sharedPrefs = SharedPreferences(userDefaults: mockDefaults)

        #expect(sharedPrefs.currency == Currency.usd.rawValue)

        sharedPrefs.currency = Currency.jpy.rawValue
        #expect(sharedPrefs.currency == Currency.jpy.rawValue)
        #expect(mockDefaults.string(forKey: Preferences.Keys.currency) == Currency.jpy.rawValue)
    }
}
