package com.gemwallet.android.domains.asset

import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class AssetDefaultsTest {

    @Test
    fun defaultBasic_nativeAsset_usesChainDefaults() {
        val asset = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9)

        val basic = asset.defaultBasic

        assertTrue(basic.properties.isEnabled)
        assertTrue(basic.properties.isSwapable)
        assertTrue(basic.properties.isStakeable)
        assertFalse(basic.properties.isBuyable)
        assertFalse(basic.properties.isSellable)
        assertFalse(basic.properties.hasImage)
    }

    @Test
    fun defaultBasic_tokenAsset_isNeverStakeable() {
        val asset = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL)

        val basic = asset.defaultBasic

        assertFalse(basic.properties.isStakeable)
        assertFalse(basic.properties.hasImage)
    }

    @Test
    fun defaultBasic_negativeRankNativeAsset_isDisabled() {
        val basic = mockAsset(id = mockAssetId(chain = Chain.Tempo)).defaultBasic

        assertEquals(-1, basic.score.rank)
        assertFalse(basic.properties.isEnabled)
        assertFalse(basic.properties.isSwapable)
    }
}
