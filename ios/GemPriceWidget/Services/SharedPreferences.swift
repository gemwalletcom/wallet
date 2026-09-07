// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

private enum SharedPreferenceConstants {
    static let currencyKey = "currency"
}

public struct SharedPreferences {
    private let userDefaults: UserDefaults?

    public init() {
        userDefaults = UserDefaults(suiteName: Constants.appGroupIdentifier)
    }

    public init(userDefaults: UserDefaults?) {
        self.userDefaults = userDefaults
    }

    public var currency: String {
        get {
            userDefaults?.string(forKey: SharedPreferenceConstants.currencyKey) ?? Currency.usd.rawValue
        }
        set {
            userDefaults?.set(newValue, forKey: SharedPreferenceConstants.currencyKey)
            userDefaults?.synchronize()
        }
    }
}
