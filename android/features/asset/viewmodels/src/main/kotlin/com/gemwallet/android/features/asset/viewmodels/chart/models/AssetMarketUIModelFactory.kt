package com.gemwallet.android.features.asset.viewmodels.chart.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.asset.viewmodels.localization.stringRes
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.property.linkRows
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import dagger.hilt.android.qualifiers.ApplicationContext
import java.math.BigDecimal
import javax.inject.Inject
import uniffi.gemstone.GemAssetMarketRow
import uniffi.gemstone.GemChartSection
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.socialLinks

class AssetMarketUIModelFactory @Inject constructor(@ApplicationContext private val context: Context) {

    fun create(asset: Asset, currency: Currency, sections: List<GemChartSection>): AssetMarketUIModel {
        val mapper = RowMapper(context, asset, currency)
        return AssetMarketUIModel(
            chain = asset.chain,
            currency = currency,
            sections = sections.map { section ->
                when (section) {
                    is GemChartSection.PriceAlerts -> ChartSectionUIModel.PriceAlerts(
                        ListItemModel(title = context.getString(R.string.settings_price_alerts_title), subtitle = section.count.toString()),
                    )
                    GemChartSection.SetPriceAlert -> ChartSectionUIModel.SetPriceAlert(ListItemModel(title = context.getString(R.string.price_alerts_set_alert_title)))
                    is GemChartSection.Market -> ChartSectionUIModel.Market(section.rows.map(mapper::row))
                    is GemChartSection.Links -> ChartSectionUIModel.Links(
                        title = context.getString(R.string.social_links),
                        links = socialLinks(section.links).linkRows(context),
                    )
                }
            },
        )
    }

    private class RowMapper(private val context: Context, private val asset: Asset, private val currency: Currency) {
        private val currencyFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = currency)
        private val supplyFormatter = ValueFormatter(style = GemValueStyle.SHORT)

        fun row(row: GemAssetMarketRow): MarketRowUIModel = when (row) {
            is GemAssetMarketRow.MarketCap -> MarketInfoUIModel(
                model = ListItemModel(title = title(row), subtitle = currencyFormatter.string(row.value), titleTag = row.rank?.let { "#$it" }),
                layout = MarketInfoUIModel.Layout.Badge,
            )
            is GemAssetMarketRow.FullyDilutedValuation -> MarketInfoUIModel(
                model = ListItemModel(title = title(row), subtitle = currencyFormatter.string(row.value), info = InfoSheetEntity.FullyDilutedValuation),
            )
            is GemAssetMarketRow.TradingVolume -> MarketInfoUIModel(
                model = ListItemModel(title = title(row), subtitle = currencyFormatter.string(row.value)),
            )
            is GemAssetMarketRow.Contract -> MarketInfoUIModel(
                model = ListItemModel(title = title(row), subtitle = row.tokenId),
                layout = MarketInfoUIModel.Layout.Address,
                explorerLink = row.explorer?.toPrimitives(),
            )
            is GemAssetMarketRow.CirculatingSupply -> MarketInfoUIModel(
                model = ListItemModel(title = title(row), subtitle = supply(row.value), info = InfoSheetEntity.CirculatingSupply),
            )
            is GemAssetMarketRow.TotalSupply -> MarketInfoUIModel(
                model = ListItemModel(title = title(row), subtitle = supply(row.value), info = InfoSheetEntity.TotalSupply),
            )
            is GemAssetMarketRow.MaxSupply -> MarketInfoUIModel(
                model = ListItemModel(title = title(row), subtitle = supply(row.value), info = InfoSheetEntity.MaxSupply),
            )
            is GemAssetMarketRow.AllTimeHigh -> row.value.toPrimitives().let {
                AllTimeUIModel.High(it.date, it.value.toDouble(), it.percentage.toDouble(), allTimeListItem(context, currency, true, it.date, it.value.toDouble(), it.percentage.toDouble()))
            }
            is GemAssetMarketRow.AllTimeLow -> row.value.toPrimitives().let {
                AllTimeUIModel.Low(it.date, it.value.toDouble(), it.percentage.toDouble(), allTimeListItem(context, currency, false, it.date, it.value.toDouble(), it.percentage.toDouble()))
            }
        }

        private fun title(row: GemAssetMarketRow): String = context.getString(row.stringRes())

        private fun supply(value: Double): String = supplyFormatter.string(BigDecimal.valueOf(value), asset.symbol)
    }
}
