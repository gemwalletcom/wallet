package com.gemwallet.android.ext

import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.asset.getSupportIconUrl
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemImage

class ChainExtTest {
    @Test
    fun seiEvm_usesEvmMappings() {
        assertEquals(AssetType.ERC20, Chain.SeiEvm.assetType())
        assertEquals("file:///android_asset/chains/icons/sei.svg", Chain.SeiEvm.getIconUrl())
    }

    @Test
    fun robinhoodNativeAsset_usesEthereumIconAndRobinhoodSupportIcon() {
        val assetId = AssetId(Chain.Robinhood)

        assertEquals("file:///android_asset/chains/icons/ethereum.svg", assetId.getIconUrl())
        assertEquals("file:///android_asset/chains/icons/robinhood.svg", assetId.getSupportIconUrl())
    }

    @Test
    fun baseDrawsItsOwnLogo_andItsTokensBadgeWithBase() {
        val usdc = AssetId(Chain.Base, "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913")
        assertEquals("file:///android_asset/chains/icons/base.svg", Chain.Base.getIconUrl())
        assertEquals("file:///android_asset/chains/icons/ethereum.svg", AssetId(Chain.Base).getIconUrl())
        assertEquals("file:///android_asset/chains/icons/base.svg", AssetId(Chain.Base).getSupportIconUrl())
        assertEquals(GemImage.Asset(usdc.toIdentifier()).url(), usdc.getIconUrl())
        assertEquals("file:///android_asset/chains/icons/base.svg", usdc.getSupportIconUrl())
        assertEquals("file:///android_asset/chains/icons/ethereum.svg", AssetId(Chain.Ethereum, "0xdac17f958d2ee523a2206206994597c13d831ec7").getSupportIconUrl())
        assertNull(AssetId(Chain.Ethereum).getSupportIconUrl())
    }
}
