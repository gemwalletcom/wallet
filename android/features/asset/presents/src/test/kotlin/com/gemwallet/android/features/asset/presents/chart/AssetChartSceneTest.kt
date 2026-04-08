package com.gemwallet.android.features.asset.presents.chart

import com.gemwallet.android.features.asset.viewmodels.chart.models.MarketInfoUIModel
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertNotNull
import org.junit.Test

class AssetChartSceneTest {

    @Test
    fun `contract market info is available for token assets without market data`() {
        val asset = mockAssetSolanaUSDC()

        val info = contractMarketInfo(asset, "Solscan") { _, _, _ -> null }

        assertNotNull(info)
        assertEquals(MarketInfoUIModel.MarketInfoTypeUIModel.Contract, info?.type)
        assertEquals(asset.id.tokenId, info?.value)
    }

    @Test
    fun `contract market info is absent for native assets`() {
        val asset = mockAssetSolana()

        val info = contractMarketInfo(asset, "Solscan") { _, _, _ -> null }

        assertNull(info)
    }
}
