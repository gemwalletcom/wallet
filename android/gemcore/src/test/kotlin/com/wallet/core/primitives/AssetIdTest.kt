package com.wallet.core.primitives

import com.gemwallet.android.ext.toAssetId
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

class AssetIdTest {

    @Test
    fun parsesIdentifier() {
        assertThrows(IllegalArgumentException::class.java) { AssetId("") }
        assertThrows(IllegalArgumentException::class.java) { AssetId("random_chain") }
        assertThrows(IllegalArgumentException::class.java) { AssetId("ethereum_") }
        assertEquals(AssetId(Chain.Bitcoin), AssetId("bitcoin"))
        assertEquals(AssetId(Chain.Ethereum, "0x123"), AssetId("ethereum_0x123"))
    }

    @Test
    fun keepsUnderscoresInsideTonTokenId() {
        val assetId = "ton_EQAvlWFDxGF2lXm67y4yzC17wYKD9A0guwPkMs1gOsM__NOT".toAssetId()!!
        assertEquals(Chain.Ton, assetId.chain)
        assertEquals("EQAvlWFDxGF2lXm67y4yzC17wYKD9A0guwPkMs1gOsM__NOT", assetId.tokenId)
    }
}
