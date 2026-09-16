package com.wallet

import com.gemwallet.android.math.parseInputNumber
import com.gemwallet.android.math.parseInputNumberOrNull
import com.gemwallet.android.math.plainInputNumber
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.math.BigDecimal
import java.text.DecimalFormatSymbols

class TestNumberParse {

    private val separator = DecimalFormatSymbols.getInstance().decimalSeparator

    @Test
    fun testTypedNumberParsesInTheDeviceLocale() {
        assertEquals(BigDecimal("0.1"), "0${separator}1".parseInputNumber())
        assertEquals(BigDecimal("1"), "1".parseInputNumber())
        assertEquals(BigDecimal("1.13"), "1${separator}13".parseInputNumber())
        assertEquals(BigDecimal("730.1234567"), "730${separator}1234567".parseInputNumber())
    }

    @Test
    fun testPlainNumberLeavesNothingWhenNoNumberWasTyped() {
        assertEquals("", "".plainInputNumber())
        assertEquals("", "abc".plainInputNumber())
        assertNull("abc".parseInputNumberOrNull())
    }
}
