package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.property.toSocialLinks
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemAssetMarketRow
import uniffi.gemstone.GemChartSection
import java.math.BigDecimal
import javax.inject.Inject

class AssetMarketUIModelFactory @Inject constructor() {

    fun create(asset: Asset, currency: Currency, sections: List<GemChartSection>): AssetMarketUIModel {
        val mapper = RowMapper(asset, currency)
        return AssetMarketUIModel(
            chain = asset.chain,
            currency = currency,
            sections = sections.map { section ->
                when (section) {
                    is GemChartSection.PriceAlerts -> ChartSectionUIModel.PriceAlerts(section.count.toInt())
                    GemChartSection.SetPriceAlert -> ChartSectionUIModel.SetPriceAlert
                    is GemChartSection.Market -> ChartSectionUIModel.Market(section.rows.map(mapper::row))
                    is GemChartSection.Links -> ChartSectionUIModel.Links(section.links.map { it.toPrimitives() }.toSocialLinks())
                }
            },
        )
    }

    private class RowMapper(private val asset: Asset, currency: Currency) {
        private val currencyFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = currency)
        private val supplyFormatter = ValueFormatter(style = ValueFormatter.Style.Short)

        fun row(row: GemAssetMarketRow): MarketRowUIModel = when (row) {
            is GemAssetMarketRow.MarketCap -> MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.MarketCap,
                value = currencyFormatter.string(row.value),
                badge = row.rank?.let { "#$it" },
            )
            is GemAssetMarketRow.FullyDilutedValuation -> MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.FDV,
                value = currencyFormatter.string(row.value),
                info = InfoSheetEntity.FullyDilutedValuation,
            )
            is GemAssetMarketRow.TradingVolume -> MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.TradingVolume,
                value = currencyFormatter.string(row.value),
            )
            is GemAssetMarketRow.Contract -> MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.Contract,
                value = row.tokenId,
                explorerLink = row.explorer?.toPrimitives(),
            )
            is GemAssetMarketRow.CirculatingSupply -> MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.CirculatingSupply,
                value = supply(row.value),
                info = InfoSheetEntity.CirculatingSupply,
            )
            is GemAssetMarketRow.TotalSupply -> MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.TotalSupply,
                value = supply(row.value),
                info = InfoSheetEntity.TotalSupply,
            )
            is GemAssetMarketRow.MaxSupply -> MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.MaxSupply,
                value = supply(row.value),
                info = InfoSheetEntity.MaxSupply,
            )
            is GemAssetMarketRow.AllTimeHigh -> row.value.toPrimitives().let { AllTimeUIModel.High(it.date, it.value.toDouble(), it.percentage.toDouble()) }
            is GemAssetMarketRow.AllTimeLow -> row.value.toPrimitives().let { AllTimeUIModel.Low(it.date, it.value.toDouble(), it.percentage.toDouble()) }
        }

        private fun supply(value: Double): String = supplyFormatter.string(BigDecimal.valueOf(value), asset.symbol)
    }
}
