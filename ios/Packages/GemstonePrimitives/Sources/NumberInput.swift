// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemNumberFormat

public enum NumberInput {
    public static func format(_ locale: Locale = .current) -> GemNumberFormat {
        GemNumberFormat(decimalSeparator: locale.decimalSeparator ?? ".")
    }

    public static func plain(_ text: String, locale: Locale = .current) -> String {
        format(locale).plain(input: text)
    }

    public static func double(_ text: String, locale: Locale = .current) -> Double? {
        Double(plain(text, locale: locale))
    }
}
