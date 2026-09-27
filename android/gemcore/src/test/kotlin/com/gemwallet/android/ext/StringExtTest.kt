package com.gemwallet.android.ext

import org.junit.Assert.assertEquals
import org.junit.Test

class StringExtTest {

    @Test
    fun `boldMarkdown wraps string in double asterisks`() {
        assertEquals("**hello**", "hello".boldMarkdown())
    }
}
