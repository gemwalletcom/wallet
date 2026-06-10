// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PreferencesTestKit
import Primitives
@testable import SharedPreferences
import Testing

struct SharedPreferencesTests {
    @Test
    func readWriteCurrency() {
        let mockDefaults = UserDefaults.mock()
        var sharedPrefs = SharedPreferences(userDefaults: mockDefaults)

        #expect(sharedPrefs.currency == Currency.usd.rawValue)

        sharedPrefs.currency = Currency.jpy.rawValue
        #expect(sharedPrefs.currency == Currency.jpy.rawValue)
        #expect(mockDefaults.string(forKey: "currency") == Currency.jpy.rawValue)
    }
}
