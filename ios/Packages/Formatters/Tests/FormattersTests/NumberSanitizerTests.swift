// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Testing

struct NumberSanitizerTests {
    @Test
    func sanitize_validNumber_shouldRemainUnchanged() {
        let sanitizer = NumberSanitizer(decimalSeparator: ".")
        #expect(sanitizer.sanitize("123.45") == "123.45")
    }

    @Test
    func sanitize_multipleDecimalSeparators_shouldKeepFirstOnly() {
        let sanitizer = NumberSanitizer(decimalSeparator: ".")
        #expect(sanitizer.sanitize("123.45.67") == "123.4567")
    }

    @Test
    func sanitize_nonNumericCharacters_shouldRemoveThem() {
        let sanitizer = NumberSanitizer(decimalSeparator: ".")
        #expect(sanitizer.sanitize("abc123.45xyz") == "123.45")
    }

    @Test
    func sanitize_whitespace_shouldBeRemoved() {
        let sanitizer = NumberSanitizer(decimalSeparator: ".")
        #expect(sanitizer.sanitize("  1 23 . 4 5 ") == "123.45")
    }

    @Test
    func sanitize_symbols_shouldBeRemoved() {
        let sanitizer = NumberSanitizer(decimalSeparator: ".")
        #expect(sanitizer.sanitize("$123.45€") == "123.45")
    }

    @Test
    func sanitize_differentDecimalSeparator_shouldBeUsed() {
        let sanitizer = NumberSanitizer(decimalSeparator: ",")
        #expect(sanitizer.sanitize("123,45,67") == "123,4567")
    }

    @Test
    func sanitize_emptyString_shouldReturnEmptyString() {
        let sanitizer = NumberSanitizer(decimalSeparator: ".")
        #expect(sanitizer.sanitize("") == "")
    }

    @Test
    func sanitize_maximumFractionDigits_shouldTruncateDecimals() {
        let sanitizer = NumberSanitizer(decimalSeparator: ".", maximumFractionDigits: 2)
        #expect(sanitizer.sanitize("0.111111111") == "0.11")
        #expect(sanitizer.sanitize("12.5") == "12.5")
        #expect(sanitizer.sanitize("12") == "12")
    }

    @Test
    func sanitize_maximumIntegerDigits_shouldTruncateInteger() {
        let sanitizer = NumberSanitizer(decimalSeparator: ".", maximumIntegerDigits: 2)
        #expect(sanitizer.sanitize("33333312312") == "33")
        #expect(sanitizer.sanitize("5") == "5")
        #expect(sanitizer.sanitize("19.555") == "19.555")
    }
}
