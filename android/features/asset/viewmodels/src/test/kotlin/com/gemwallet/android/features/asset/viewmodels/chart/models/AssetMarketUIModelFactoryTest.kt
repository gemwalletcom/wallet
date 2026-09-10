package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetLink
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.ChartValuePercentage
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemAssetMarketRow
import uniffi.gemstone.GemChartSection

class AssetMarketUIModelFactoryTest {

    private val asset = mockAssetSolanaUSDC()
    private val factory = AssetMarketUIModelFactory()

    @Test
    fun `rows keep their section and the rank badge core gives`() {
        val tokenId = requireNotNull(asset.id.tokenId)
        val explorer = BlockExplorerLink(name = "Solscan", link = "https://solscan.io/token/$tokenId")
        val allTimeHigh = ChartValuePercentage(date = 10L, value = 1.5f, percentage = -5f)
        val sections = listOf(
            GemChartSection.PriceAlerts(count = 2u),
            GemChartSection.Market(listOf(GemAssetMarketRow.MarketCap(value = 1.0, rank = 7), GemAssetMarketRow.TradingVolume(value = 2.0))),
            GemChartSection.Market(listOf(GemAssetMarketRow.Contract(tokenId = tokenId, explorer = explorer.toGem()))),
            GemChartSection.Market(listOf(GemAssetMarketRow.CirculatingSupply(value = 3.0))),
            GemChartSection.Market(listOf(GemAssetMarketRow.AllTimeHigh(value = allTimeHigh.toGem()))),
            GemChartSection.Links(listOf(mockAssetLink().toGem())),
        )

        val model = factory.create(asset, Currency.USD, sections)

        assertEquals(ChartSectionUIModel.PriceAlerts(2), model.sections[0])
        val marketRows = (model.sections[1] as ChartSectionUIModel.Market).rows.map { it as MarketInfoUIModel }
        assertEquals(
            listOf(MarketInfoUIModel.MarketInfoTypeUIModel.MarketCap, MarketInfoUIModel.MarketInfoTypeUIModel.TradingVolume),
            marketRows.map { it.type },
        )
        assertEquals("#7", marketRows.first().badge)
        assertNull(marketRows.last().badge)
        val contract = (model.sections[2] as ChartSectionUIModel.Market).rows.single() as MarketInfoUIModel
        assertEquals(tokenId, contract.value)
        assertEquals(explorer, contract.explorerLink)
        assertEquals(
            listOf(MarketInfoUIModel.MarketInfoTypeUIModel.CirculatingSupply),
            (model.sections[3] as ChartSectionUIModel.Market).rows.map { (it as MarketInfoUIModel).type },
        )
        val high = (model.sections[4] as ChartSectionUIModel.Market).rows.single() as AllTimeUIModel.High
        assertEquals(1.5, high.value, 0.0)
        assertEquals(-5.0, high.percentage, 0.0)
        assertEquals(1, (model.sections[5] as ChartSectionUIModel.Links).links.size)
    }

    @Test
    fun `supply rows carry the asset symbol`() {
        val sections = listOf(
            GemChartSection.Market(listOf(GemAssetMarketRow.CirculatingSupply(value = 1500.0), GemAssetMarketRow.MaxSupply(value = 21.0))),
        )

        val model = factory.create(asset, Currency.USD, sections)

        assertEquals(
            listOf("1,500 ${asset.symbol}", "21 ${asset.symbol}"),
            (model.sections.single() as ChartSectionUIModel.Market).rows.map { (it as MarketInfoUIModel).value },
        )
    }
}
