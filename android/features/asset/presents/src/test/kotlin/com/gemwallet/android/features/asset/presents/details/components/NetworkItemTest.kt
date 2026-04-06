package com.gemwallet.android.features.asset.presents.details.components

import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.ui.models.actions.AssetIdAction
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertSame
import org.junit.Test

class NetworkItemTest {

    @Test
    fun `native assets keep network row but disable navigation`() {
        val asset = mockAssetEthereum()
        val uiModel = AssetInfoUIModel(
            assetInfo = mockAssetInfo(asset = asset),
            tokenType = asset.type,
        )

        val rowState = uiModel.networkRowState(OPEN_NETWORK)

        assertEquals(asset, rowState.asset)
        assertNull(rowState.onOpenNetwork)
    }

    @Test
    fun `token assets keep network row and allow navigation`() {
        val asset = mockAssetSolanaUSDC()
        val uiModel = AssetInfoUIModel(
            assetInfo = mockAssetInfo(asset = asset),
            tokenType = asset.type,
        )

        val rowState = uiModel.networkRowState(OPEN_NETWORK)

        assertEquals(asset, rowState.asset)
        assertSame(OPEN_NETWORK, rowState.onOpenNetwork)
    }

    private companion object {
        val OPEN_NETWORK = AssetIdAction { }
    }
}
