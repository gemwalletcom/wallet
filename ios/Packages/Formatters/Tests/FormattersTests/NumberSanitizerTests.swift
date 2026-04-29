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
}
