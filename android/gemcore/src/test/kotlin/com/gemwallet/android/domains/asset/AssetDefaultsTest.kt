package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.isStakeSupported
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAssetConfigService

class AssetDefaultsTest {

    private val assetConfig = GemAssetConfigService()

    @Test
    fun defaultBasic_nativeAsset_usesChainDefaults() {
        val asset = mockAssetSolana()

        val basic = asset.defaultBasic

        assertEquals(assetConfig.defaultRank(Chain.Solana.string), basic.score.rank)
        assertTrue(basic.properties.isEnabled)
        assertEquals(assetConfig.isSwapable(asset.id.toIdentifier()), basic.properties.isSwapable)
        assertEquals(Chain.Solana.isStakeSupported(), basic.properties.isStakeable)
        assertFalse(basic.properties.isBuyable)
        assertFalse(basic.properties.isSellable)
        assertFalse(basic.properties.hasImage)
    }

    @Test
    fun defaultBasic_tokenAsset_isNeverStakeable() {
        val asset = mockAssetSolanaUSDC()

        val basic = asset.defaultBasic

        assertEquals(assetConfig.defaultRank(asset.id.toIdentifier()), basic.score.rank)
        assertFalse(basic.properties.isStakeable)
        assertFalse(basic.properties.hasImage)
    }

    @Test
    fun defaultBasic_negativeRankNativeAsset_isDisabled() {
        val basic = mockAsset(chain = Chain.Tempo).defaultBasic

        assertEquals(-1, basic.score.rank)
        assertFalse(basic.properties.isEnabled)
        assertFalse(basic.properties.isSwapable)
    }
}
