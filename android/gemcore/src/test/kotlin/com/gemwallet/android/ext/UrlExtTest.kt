package com.gemwallet.android.ext

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class UrlExtTest {

    @Test
    fun getShortUrl_returnsNormalizedHost() {
        assertEquals("venice.ai", "https://www.venice.ai/path?query=1#fragment".getShortUrl())
        assertEquals("venice.ai", "http://www.venice.ai".getShortUrl())
        assertEquals("venice.ai", "www.venice.ai/path".getShortUrl())
        assertEquals("venice.ai", "venice.ai".getShortUrl())
        assertNull(" ".getShortUrl())
    }

    @Test
    fun walletConnectAppName_fallsBackToNormalizedHost() {
        assertEquals("Venice.ai", walletConnectAppName("Venice.ai", "https://www.venice.ai"))
        assertEquals("venice.ai", walletConnectAppName("", "https://www.venice.ai/path"))
        assertEquals("", walletConnectAppName("", ""))
    }
}
