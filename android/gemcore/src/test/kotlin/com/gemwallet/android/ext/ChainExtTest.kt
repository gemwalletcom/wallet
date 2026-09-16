package com.gemwallet.android.ext

import com.gemwallet.android.domains.asset.iconChain
import com.gemwallet.android.domains.asset.remoteIconUrl
import com.gemwallet.android.domains.asset.supportIconChain
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
        assertEquals(Chain.Sei, Chain.SeiEvm.iconChain())
    }

    @Test
    fun robinhoodNativeAsset_usesEthereumIconAndRobinhoodSupportIcon() {
        val assetId = AssetId(Chain.Robinhood)

        assertEquals(Chain.Ethereum, assetId.iconChain())
        assertEquals(Chain.Robinhood, assetId.supportIconChain())
    }

    @Test
    fun baseDrawsItsOwnLogo_andItsTokensBadgeWithBase() {
        val usdc = AssetId(Chain.Base, "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913")
        assertEquals(Chain.Base, Chain.Base.iconChain())
        assertEquals(Chain.Ethereum, AssetId(Chain.Base).iconChain())
        assertEquals(Chain.Base, AssetId(Chain.Base).supportIconChain())
        assertNull(usdc.iconChain())
        assertEquals(GemImage.Asset(usdc.toIdentifier()).url(), usdc.remoteIconUrl())
        assertEquals(Chain.Base, usdc.supportIconChain())
        assertEquals(Chain.Ethereum, AssetId(Chain.Ethereum, "0xdac17f958d2ee523a2206206994597c13d831ec7").supportIconChain())
        assertNull(AssetId(Chain.Ethereum).supportIconChain())
    }
}
