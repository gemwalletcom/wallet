package com.gemwallet.android.math

import org.junit.Assert.assertEquals
import org.junit.Test

class NumberSanitizerTest {
    @Test
    fun sanitize_removesNonNumericCharacters() {
        assertEquals("123.45", NumberSanitizer().sanitize("abc123.45xyz"))
    }

    @Test
    fun sanitize_keepsFirstSeparatorOnly() {
        assertEquals("123.4567", NumberSanitizer().sanitize("123.45.67"))
    }

    @Test
    fun sanitize_limitsFractionDigits() {
        val sanitizer = NumberSanitizer(maximumFractionDigits = 2)
        assertEquals("0.11", sanitizer.sanitize("0.111111"))
        assertEquals("12.5", sanitizer.sanitize("12.5"))
        assertEquals("12", sanitizer.sanitize("12"))
    }

    @Test
    fun sanitize_limitsIntegerDigits() {
        val sanitizer = NumberSanitizer(maximumIntegerDigits = 2)
        assertEquals("33", sanitizer.sanitize("33333312312"))
        assertEquals("5", sanitizer.sanitize("5"))
        assertEquals("19.555", sanitizer.sanitize("19.555"))
    }
}
