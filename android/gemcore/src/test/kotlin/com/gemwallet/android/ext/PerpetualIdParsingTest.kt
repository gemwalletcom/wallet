package com.gemwallet.android.ext

import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class PerpetualIdParsingTest {

    @Test
    fun parseAndFormat() {
        val hypercoreBtc = "hypercore_BTC".toPerpetualId()
        assertEquals(PerpetualId(PerpetualProvider.Hypercore, "BTC"), hypercoreBtc)
        assertEquals("hypercore_BTC", hypercoreBtc?.toIdentifier())

        val symbolWithUnderscore = "hypercore_BTC_PERP".toPerpetualId()
        assertEquals(PerpetualId(PerpetualProvider.Hypercore, "BTC_PERP"), symbolWithUnderscore)
        assertEquals("hypercore_BTC_PERP", symbolWithUnderscore?.toIdentifier())

        assertNull("hypercore".toPerpetualId())
        assertNull("unknownprovider_BTC".toPerpetualId())
        assertNull("_BTC".toPerpetualId())
        assertNull("".toPerpetualId())
    }
}
