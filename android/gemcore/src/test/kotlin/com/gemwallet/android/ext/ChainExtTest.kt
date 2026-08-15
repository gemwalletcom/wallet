package com.gemwallet.android.ext

import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.asset.getSupportIconUrl
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainType
import com.wallet.core.primitives.EVMChain
import org.junit.Assert.assertEquals
import org.junit.Test

class ChainExtTest {
    @Test
    fun seiEvm_usesEvmMappings() {
        assertEquals(AssetType.ERC20, Chain.SeiEvm.assetType())
        assertEquals(EVMChain.SeiEvm, Chain.SeiEvm.toEVM())
        assertEquals(ChainType.Ethereum, Chain.SeiEvm.toChainType())
        assertEquals("file:///android_asset/chains/icons/sei.svg", Chain.SeiEvm.getIconUrl())
    }

    @Test
    fun robinhoodNativeAsset_usesEthereumIconAndRobinhoodSupportIcon() {
        val assetId = AssetId(Chain.Robinhood)

        assertEquals("file:///android_asset/chains/icons/ethereum.svg", assetId.getIconUrl())
        assertEquals("file:///android_asset/chains/icons/robinhood.svg", assetId.getSupportIconUrl())
    }
}
