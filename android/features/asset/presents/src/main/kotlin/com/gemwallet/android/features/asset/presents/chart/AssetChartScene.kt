package com.gemwallet.android.features.asset.presents.chart

import androidx.compose.foundation.clickable
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.toPriceState
import com.gemwallet.android.ext.AddressFormatter
import com.gemwallet.android.model.compactFormatter
import com.gemwallet.android.model.formatSupply
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.components.list_item.ChipBadge
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.features.asset.viewmodels.chart.models.AllTimeUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetMarketUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.MarketInfoUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.AssetChartViewModel
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.ChartViewModel
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.Currency
import uniffi.gemstone.Explorer
import java.text.DateFormat
import java.util.Date

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AssetChartScene(
    onCancel: () -> Unit,
    onPriceAlerts: (AssetId) -> Unit,
    onAddPriceAlertTarget: (AssetId) -> Unit,
    viewModel: AssetChartViewModel = hiltViewModel(),
    chartViewModel: ChartViewModel = hiltViewModel(),
) {
    val marketUIModelState by viewModel.marketUIModel.collectAsStateWithLifecycle()
    val priceAlertsCount by viewModel.priceAlertsCount.collectAsStateWithLifecycle()
    val isChartRefreshing by chartViewModel.isRefreshing.collectAsStateWithLifecycle()
    val pullToRefreshState = rememberPullToRefreshState()

    val marketModel = marketUIModelState
    if (marketModel == null) {
        LoadingScene(stringResource(R.string.common_loading), onCancel)
        return
    }

    Scene(
        title = marketModel.assetTitle,
        backHandle = true,
        onClose = onCancel,
    ) {
        PullToRefreshBox(
            isRefreshing = isChartRefreshing,
            onRefresh = {
                chartViewModel.refresh()
            },
            state = pullToRefreshState,
            indicator = {
                PullToRefreshDefaults.Indicator(
                    modifier = Modifier.align(Alignment.TopCenter),
                    isRefreshing = isChartRefreshing,
                    state = pullToRefreshState,
                )
            },
        ) {
            LazyColumn {
                item { Chart(chartViewModel) }
                item {
                    PriceAlertsItem(
                        assetId = marketModel.asset.id,
                        priceAlertsCount = priceAlertsCount,
                        onPriceAlerts = onPriceAlerts,
                        onAddPriceAlertTarget = onAddPriceAlertTarget,
                    )
                }
                assetMarket(marketModel.currency, marketModel.asset, marketModel.marketInfo, marketModel.explorerName)
                links(marketModel.assetLinks)
            }
        }
    }
}

@Composable
private fun PriceAlertsItem(
    assetId: AssetId,
    priceAlertsCount: Int,
    onPriceAlerts: (AssetId) -> Unit,
    onAddPriceAlertTarget: (AssetId) -> Unit,
) {
    val hasPriceAlerts = priceAlertsCount > 0

    PropertyItem(
        modifier = Modifier
            .clickable {
                if (hasPriceAlerts) {
                    onPriceAlerts(assetId)
                } else {
                    onAddPriceAlertTarget(assetId)
                }
            }
            .testTag("assetChart"),
        title = {
            PropertyTitleText(
                if (hasPriceAlerts) {
                    R.string.settings_price_alerts_title
                } else {
                    R.string.price_alerts_set_alert_title
                }
            )
        },
        data = {
            PropertyDataText(
                text = if (hasPriceAlerts) priceAlertsCount.toString() else "",
                badge = { DataBadgeChevron() },
            )
        },
        listPosition = ListPosition.Single,
    )
}

private fun LazyListScope.links(links: List<AssetMarketUIModel.Link>) {
    if (links.isEmpty()) return
    item { SubheaderItem(R.string.social_links) }
    itemsIndexed(links) { index, item ->
        val uriHandler = LocalUriHandler.current
        val context = LocalContext.current
        PropertyItem(
            modifier = Modifier.clickable { uriHandler.open(context, item.url) },
            title = { PropertyTitleText(item.label, trailing = { AsyncImage(item.icon, smallIconSize) }) },
            data = { PropertyDataText("", badge = { DataBadgeChevron() }) },
            listPosition = ListPosition.getPosition(index, links.size)
        )
    }
}

private fun LazyListScope.assetMarket(currency: Currency, asset: Asset, marketInfo: AssetMarket?, explorerName: String) {
    marketInfo ?: return
    val marketItems = listOfNotNull(
        asset.id.tokenId?.let {
            MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.Contract,
                value = it,
            )
        },
        marketInfo.marketCap?.let {
            MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.MarketCap,
                value = currency.compactFormatter(it),
                badge = marketInfo.marketCapRank?.takeIf { rank -> rank > 0 }?.let { "#$it" },
            )
        },
        marketInfo.totalVolume?.let {
            MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.TradingVolume,
                value = currency.compactFormatter(it),
            )
        },
        marketInfo.marketCapFdv?.let {
            MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.FDV,
                value = currency.compactFormatter(it),
                info = InfoSheetEntity.FullyDilutedValuation,
            )
        },
    )

    val supplyItems = listOfNotNull(
        marketInfo.circulatingSupply?.let {
            MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.CirculatingSupply,
                value = asset.compactFormatter(it),
                info = InfoSheetEntity.CirculatingSupply,
            )
        },
        marketInfo.totalSupply?.let {
            MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.TotalSupply,
                value = asset.compactFormatter(it),
                info = InfoSheetEntity.TotalSupply,
            )
        },
        marketInfo.maxSupply?.let {
            MarketInfoUIModel(
                type = MarketInfoUIModel.MarketInfoTypeUIModel.MaxSupply,
                value = asset.formatSupply(it),
                info = InfoSheetEntity.MaxSupply,
            )
        },
    )

    val allTime = listOfNotNull(
        marketInfo.allTimeHighValue?.let { AllTimeUIModel.High(it.date, it.value.toDouble(), it.percentage.toDouble()) },
        marketInfo.allTimeLowValue?.let { AllTimeUIModel.Low(it.date, it.value.toDouble(), it.percentage.toDouble()) },
    )

    marketProperties(asset, explorerName, marketItems)
    marketProperties(asset, explorerName, supplyItems)
    allTimeProperties(asset, currency, allTime)
}

private fun LazyListScope.marketProperties(asset: Asset, explorerName: String, items: List<MarketInfoUIModel>) {
    itemsPositioned(items) { position, item ->
        when (item.type) {
            MarketInfoUIModel.MarketInfoTypeUIModel.FDV,
            MarketInfoUIModel.MarketInfoTypeUIModel.TradingVolume,
            MarketInfoUIModel.MarketInfoTypeUIModel.CirculatingSupply,
            MarketInfoUIModel.MarketInfoTypeUIModel.TotalSupply,
            MarketInfoUIModel.MarketInfoTypeUIModel.MaxSupply -> PropertyItem(item.type.label, item.value, listPosition = position, info = item.info)
            MarketInfoUIModel.MarketInfoTypeUIModel.MarketCap -> PropertyItem(
                title = {
                    PropertyTitleText(
                        text = item.type.label,
                        badge = item.badge?.let { { ChipBadge(it) } }
                    )
                },
                data = { PropertyDataText(item.value) },
                listPosition = position
            )
            MarketInfoUIModel.MarketInfoTypeUIModel.Contract -> {
                val context = LocalContext.current
                val clipboardManager = LocalClipboard.current.nativeClipboard
                val uriHandler = LocalUriHandler.current
                PropertyItem(
                    modifier = Modifier.combinedClickable(
                        onLongClick = {
                            clipboardManager.setPlainText(context, item.value)
                        },
                        onClick = {
                            uriHandler.open(context, Explorer(asset.chain.string).getTokenUrl(explorerName, item.value) ?: return@combinedClickable)
                        }
                    ),
                    title = { PropertyTitleText(R.string.asset_contract) },
                    data = {
                        PropertyDataText(
                            text = AddressFormatter(item.value, chain = asset.chain).value(),
                            badge = { DataBadgeChevron() }
                        )
                    },
                    listPosition = position
                )
            }
        }
    }
}

private fun LazyListScope.allTimeProperties(asset: Asset, currency: Currency, items: List<AllTimeUIModel>) {
    val dateFormat = DateFormat.getDateInstance(DateFormat.MEDIUM)

    itemsPositioned(items) { position, item ->
        val title = when (item) {
            is AllTimeUIModel.High -> R.string.asset_all_time_high
            is AllTimeUIModel.Low -> R.string.asset_all_time_low
        }
        ListItem(
            listPosition = position,
            title = { PropertyTitleText(text = stringResource(title)) },
            subtitle = { ListItemSupportText(dateFormat.format(Date(item.date))) },
            trailing = {
                val rowScope = this
                Column(horizontalAlignment = Alignment.End) {
                    with(rowScope) { PropertyDataText(currency.compactFormatter(item.value)) }
                    ListItemSupportText(item.percentage.formatAsPercentage(), color = item.percentage.toPriceState().color())
                }
            },
        )
    }

}
