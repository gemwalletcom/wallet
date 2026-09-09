package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.ChartValuePercentage
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemAssetMarketRow
import uniffi.gemstone.GemAssetMarketRows

class AssetMarketUIModelFactoryTest {

    private val asset = mockAssetSolanaUSDC()
    private val factory = AssetMarketUIModelFactory()

    @Test
    fun `rows keep their section and the rank badge core gives`() {
        val tokenId = requireNotNull(asset.id.tokenId)
        val explorer = BlockExplorerLink(name = "Solscan", link = "https://solscan.io/token/$tokenId")
        val allTimeHigh = ChartValuePercentage(date = 10L, value = 1.5f, percentage = -5f)
        val rows = GemAssetMarketRows(
            market = listOf(GemAssetMarketRow.MarketCap(value = 1.0, rank = 7), GemAssetMarketRow.TradingVolume(value = 2.0)),
            contract = listOf(GemAssetMarketRow.Contract(tokenId = tokenId, explorer = explorer.toGem())),
            supply = listOf(GemAssetMarketRow.CirculatingSupply(value = 3.0)),
            allTime = listOf(GemAssetMarketRow.AllTimeHigh(value = allTimeHigh.toGem())),
        )

        val model = factory.create(asset, Currency.USD, rows, emptyList())

        val marketRows = model.marketRows.map { it as MarketInfoUIModel }
        assertEquals(
            listOf(MarketInfoUIModel.MarketInfoTypeUIModel.MarketCap, MarketInfoUIModel.MarketInfoTypeUIModel.TradingVolume),
            marketRows.map { it.type },
        )
        assertEquals("#7", marketRows.first().badge)
        assertNull(marketRows.last().badge)
        val contract = model.contractRows.single() as MarketInfoUIModel
        assertEquals(tokenId, contract.value)
        assertEquals(explorer, contract.explorerLink)
        assertEquals(listOf(MarketInfoUIModel.MarketInfoTypeUIModel.CirculatingSupply), model.supplyRows.map { (it as MarketInfoUIModel).type })
        val high = model.allTimeRows.single() as AllTimeUIModel.High
        assertEquals(1.5, high.value, 0.0)
        assertEquals(-5.0, high.percentage, 0.0)
    }

    @Test
    fun `supply rows carry the asset symbol`() {
        val rows = GemAssetMarketRows(
            market = emptyList(),
            contract = emptyList(),
            supply = listOf(GemAssetMarketRow.CirculatingSupply(value = 1500.0), GemAssetMarketRow.MaxSupply(value = 21.0)),
            allTime = emptyList(),
        )

        val model = factory.create(asset, Currency.USD, rows, emptyList())

        assertEquals(listOf("1,500 ${asset.symbol}", "21 ${asset.symbol}"), model.supplyRows.map { (it as MarketInfoUIModel).value })
    }
}
