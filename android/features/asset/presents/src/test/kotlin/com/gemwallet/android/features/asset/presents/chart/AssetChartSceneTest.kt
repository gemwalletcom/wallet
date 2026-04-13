package com.gemwallet.android.features.asset.presents.chart

import com.gemwallet.android.features.asset.viewmodels.chart.models.MarketInfoUIModel
import com.gemwallet.android.testkit.mockAssetMarket
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

    @Test
    fun `supply items use compact formatter except max supply`() {
        val items = buildSupplyItems(
            marketInfo = mockAssetMarket(
                circulatingSupply = 1.0,
                totalSupply = 2.0,
                maxSupply = 3.0,
            ),
            compactSupplyFormatter = { "compact:$it" },
            maxSupplyFormatter = { "max:$it" },
        )

        assertEquals(
            listOf(
                MarketInfoUIModel.MarketInfoTypeUIModel.CirculatingSupply,
                MarketInfoUIModel.MarketInfoTypeUIModel.TotalSupply,
                MarketInfoUIModel.MarketInfoTypeUIModel.MaxSupply,
            ),
            items.map { it.type }
        )
        assertEquals(
            listOf("compact:1.0", "compact:2.0", "max:3.0"),
            items.map { it.value }
        )
    }

    @Test
    fun `market items use compact formatter and limit market cap badge to top 1000`() {
        val items = buildMarketItems(
            marketInfo = mockAssetMarket(
                marketCap = 1.0,
                totalVolume = 2.0,
                marketCapFdv = 3.0,
                marketCapRank = 1001,
            ),
            compactCurrencyFormatter = { "compact:$it" },
        )

        assertEquals(
            listOf(
                MarketInfoUIModel.MarketInfoTypeUIModel.MarketCap,
                MarketInfoUIModel.MarketInfoTypeUIModel.FDV,
                MarketInfoUIModel.MarketInfoTypeUIModel.TradingVolume,
            ),
            items.map { it.type }
        )
        assertEquals(
            listOf("compact:1.0", "compact:3.0", "compact:2.0"),
            items.map { it.value }
        )
        assertNull(items.first().badge)
    }
}
