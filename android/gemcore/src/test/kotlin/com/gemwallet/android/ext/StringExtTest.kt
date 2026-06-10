package com.gemwallet.android.ext

import org.junit.Assert.assertEquals
import org.junit.Test

class StringExtTest {

    @Test
    fun `boldMarkdown wraps string in double asterisks`() {
        assertEquals("**hello**", "hello".boldMarkdown())
    }

    @Test
    fun `words splits on any whitespace and drops blanks`() {
        assertEquals(listOf("alpha", "bravo", "charlie"), "alpha bravo charlie".words())
        assertEquals(listOf("alpha", "bravo", "charlie"), "  alpha\t bravo\n\ncharlie  ".words())
        assertEquals(listOf("alpha", "bravo"), "alpha\u00A0bravo".words())
        assertEquals(emptyList<String>(), "".words())
        assertEquals(emptyList<String>(), " \t\n ".words())
    }
}
