package com.gemwallet.android.math

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Test

class HexTest {

    @Test
    fun hexReturnsUnprefixedLowercaseString() {
        assertEquals("0a1fff", byteArrayOf(0x0a, 0x1f, 0xff.toByte()).hex)
    }

    @Test
    fun hexAppend0xReturnsPrefixedLowercaseString() {
        assertEquals("0x0a1fff", byteArrayOf(0x0a, 0x1f, 0xff.toByte()).hex.append0x())
    }

    @Test
    fun remove0xReturnsUnprefixedString() {
        assertEquals("0a1fff", "0x0a1fff".remove0x())
        assertEquals("0a1fff", "0a1fff".remove0x())
    }

    @Test
    fun fromHexAcceptsPrefixedAndUnprefixedStrings() {
        val bytes = byteArrayOf(0x0a, 0x1f, 0xff.toByte())

        assertArrayEquals(bytes, "0a1fff".fromHex())
        assertArrayEquals(bytes, "0x0a1fff".fromHex())
    }
}
