// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension UserDefaults {
    func preferenceValue(forKey key: String) -> String? {
        switch object(forKey: key) {
        case let value as String: value
        case let number as NSNumber: CFGetTypeID(number as CFTypeRef) == CFBooleanGetTypeID() ? String(number.boolValue) : number.stringValue
        default: .none
        }
    }
}
