package com.gemwallet.android.math

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Test

class HexTest {

    @Test
    fun encodeHexReturnsUnprefixedLowercaseString() {
        assertEquals("0a1fff", byteArrayOf(0x0a, 0x1f, 0xff.toByte()).encodeHex())
    }

    @Test
    fun encodeHexWith0xReturnsPrefixedLowercaseString() {
        assertEquals("0x0a1fff", byteArrayOf(0x0a, 0x1f, 0xff.toByte()).encodeHexWith0x())
    }

    @Test
    fun decodeHexAcceptsPrefixedAndUnprefixedStrings() {
        val bytes = byteArrayOf(0x0a, 0x1f, 0xff.toByte())

        assertArrayEquals(bytes, "0a1fff".decodeHex())
        assertArrayEquals(bytes, "0x0a1fff".decodeHex())
    }
}
