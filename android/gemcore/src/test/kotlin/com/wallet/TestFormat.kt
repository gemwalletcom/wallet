package com.wallet

import com.gemwallet.android.model.formatSupply
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import junit.framework.TestCase.assertEquals
import org.junit.Test
import java.util.Locale

class TestFormat {

    @Test
    fun testFormatSupply() {
        val btc = Asset(AssetId(Chain.Bitcoin), "Bitcoin", "BTC", 8, AssetType.NATIVE)
        assertEquals("∞ BTC", btc.formatSupply(0.0, Locale.US))
    }
}
