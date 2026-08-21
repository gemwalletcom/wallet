package com.gemwallet.android.features.asset.presents.details.components

import com.gemwallet.android.features.asset.presents.details.AssetDetailsAction
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockAssetTempoUSDCe
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test

class NetworkItemTest {

    @Test
    fun networkNavigation_usesAvailableNativeAssetOrNetworkAssets() {
        assertEquals(
            AssetDetailsAction.OpenNetwork(AssetId(Chain.Solana)),
            mockAssetSolanaUSDC().networkNavigationAction(hasNativeAsset = true),
        )
        assertEquals(
            AssetDetailsAction.OpenNetworkAssets(Chain.Tempo),
            mockAssetTempoUSDCe().networkNavigationAction(hasNativeAsset = false),
        )
        assertEquals(
            AssetDetailsAction.OpenNetworkAssets(Chain.Ethereum),
            mockAssetEthereum().networkNavigationAction(hasNativeAsset = true),
        )
    }
}
