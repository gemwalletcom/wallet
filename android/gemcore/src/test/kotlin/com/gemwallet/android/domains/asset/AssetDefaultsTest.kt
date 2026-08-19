package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.isStakeSupported
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.assetDefaultRank
import uniffi.gemstone.assetIsSwapable

class AssetDefaultsTest {

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun defaultBasic_nativeAsset_usesChainDefaults() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        mockkStatic("uniffi.gemstone.GemstoneKt")

        val asset = mockAssetSolana()
        every { assetIsSwapable(asset.id.toIdentifier()) } returns true
        every { Chain.Solana.isStakeSupported() } returns true
        every { assetDefaultRank(Chain.Solana.string) } returns 99

        val basic = asset.defaultBasic

        assertEquals(99, basic.score.rank)
        assertTrue(basic.properties.isEnabled)
        assertTrue(basic.properties.isSwapable)
        assertTrue(basic.properties.isStakeable)
        assertFalse(basic.properties.isBuyable)
        assertFalse(basic.properties.isSellable)
        assertFalse(basic.properties.hasImage)

        verify(exactly = 1) { assetDefaultRank(Chain.Solana.string) }
        verify(exactly = 1) { assetIsSwapable(asset.id.toIdentifier()) }
        verify(exactly = 1) { Chain.Solana.isStakeSupported() }
    }

    @Test
    fun defaultBasic_tokenAsset_usesTokenDefaults() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        mockkStatic("uniffi.gemstone.GemstoneKt")

        val asset = mockAssetSolanaUSDC()
        every { assetIsSwapable(asset.id.toIdentifier()) } returns false
        every { Chain.Solana.isStakeSupported() } returns true
        every { assetDefaultRank(any()) } returns 99

        val basic = asset.defaultBasic

        assertEquals(15, basic.score.rank)
        assertTrue(basic.properties.isEnabled)
        assertFalse(basic.properties.isSwapable)
        assertFalse(basic.properties.isStakeable)
        assertFalse(basic.properties.hasImage)

        verify(exactly = 0) { assetDefaultRank(any()) }
        verify(exactly = 1) { assetIsSwapable(asset.id.toIdentifier()) }
        verify(exactly = 0) { Chain.Solana.isStakeSupported() }
    }

    @Test
    fun defaultBasic_tempoNetworkAnchor_isNotSwappable() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        mockkStatic("uniffi.gemstone.GemstoneKt")
        every { assetIsSwapable(any()) } returns false
        every { Chain.Tempo.isStakeSupported() } returns false
        every { assetDefaultRank(Chain.Tempo.string) } returns 15

        val basic = mockAsset(chain = Chain.Tempo).defaultBasic

        assertFalse(basic.properties.isSwapable)
    }
}
