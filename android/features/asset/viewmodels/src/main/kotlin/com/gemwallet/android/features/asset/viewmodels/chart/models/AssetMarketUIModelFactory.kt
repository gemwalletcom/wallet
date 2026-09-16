package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.features.asset.viewmodels.localization.stringRes
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.property.toSocialLinks
import uniffi.gemstone.socialLinks
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemAssetMarketRow
import uniffi.gemstone.GemChartSection
import java.math.BigDecimal
import javax.inject.Inject
import uniffi.gemstone.GemValueStyle

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
                    is GemChartSection.Links -> ChartSectionUIModel.Links(socialLinks(section.links).toSocialLinks())
                }
            },
        )
    }

    private class RowMapper(private val asset: Asset, currency: Currency) {
        private val currencyFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = currency)
        private val supplyFormatter = ValueFormatter(style = GemValueStyle.SHORT)

        fun row(row: GemAssetMarketRow): MarketRowUIModel = when (row) {
            is GemAssetMarketRow.MarketCap -> MarketInfoUIModel(
                label = row.stringRes(),
                value = currencyFormatter.string(row.value),
                layout = MarketInfoUIModel.Layout.Badge,
                badge = row.rank?.let { "#$it" },
            )
            is GemAssetMarketRow.FullyDilutedValuation -> MarketInfoUIModel(
                label = row.stringRes(),
                value = currencyFormatter.string(row.value),
                info = InfoSheetEntity.FullyDilutedValuation,
            )
            is GemAssetMarketRow.TradingVolume -> MarketInfoUIModel(
                label = row.stringRes(),
                value = currencyFormatter.string(row.value),
            )
            is GemAssetMarketRow.Contract -> MarketInfoUIModel(
                label = row.stringRes(),
                value = row.tokenId,
                layout = MarketInfoUIModel.Layout.Address,
                explorerLink = row.explorer?.toPrimitives(),
            )
            is GemAssetMarketRow.CirculatingSupply -> MarketInfoUIModel(
                label = row.stringRes(),
                value = supply(row.value),
                info = InfoSheetEntity.CirculatingSupply,
            )
            is GemAssetMarketRow.TotalSupply -> MarketInfoUIModel(
                label = row.stringRes(),
                value = supply(row.value),
                info = InfoSheetEntity.TotalSupply,
            )
            is GemAssetMarketRow.MaxSupply -> MarketInfoUIModel(
                label = row.stringRes(),
                value = supply(row.value),
                info = InfoSheetEntity.MaxSupply,
            )
            is GemAssetMarketRow.AllTimeHigh -> row.value.toPrimitives().let { AllTimeUIModel.High(it.date, it.value.toDouble(), it.percentage.toDouble()) }
            is GemAssetMarketRow.AllTimeLow -> row.value.toPrimitives().let { AllTimeUIModel.Low(it.date, it.value.toDouble(), it.percentage.toDouble()) }
        }

        private fun supply(value: Double): String = supplyFormatter.string(BigDecimal.valueOf(value), asset.symbol)
    }
}
