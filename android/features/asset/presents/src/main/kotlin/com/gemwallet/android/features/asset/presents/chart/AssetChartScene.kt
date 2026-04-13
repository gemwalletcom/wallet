package com.gemwallet.android.features.asset.presents.chart

import androidx.compose.foundation.clickable
import com.gemwallet.android.ext.AddressFormatter
import androidx.compose.foundation.layout.Column
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.toPriceState
import com.gemwallet.android.model.compactFormatter
import com.gemwallet.android.model.format
import com.gemwallet.android.model.formatSupply
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.components.list_item.ChipBadge
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
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
import com.wallet.core.primitives.BlockExplorerLink
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
    toastMessage: String? = null,
    onToastShown: () -> Unit = {},
    viewModel: AssetChartViewModel = hiltViewModel(),
    chartViewModel: ChartViewModel = hiltViewModel(),
) {
    val marketModel by viewModel.marketUIModel.collectAsStateWithLifecycle()
    val title by viewModel.title.collectAsStateWithLifecycle()
    val priceAlertsCount by viewModel.priceAlertsCount.collectAsStateWithLifecycle()
    val isChartRefreshing by chartViewModel.isRefreshing.collectAsStateWithLifecycle()
    val pullToRefreshState = rememberPullToRefreshState()
    val snackbar = rememberSnackbarState(message = toastMessage, onShown = onToastShown)

    Scene(
        title = title,
        backHandle = true,
        onClose = onCancel,
        snackbar = snackbar,
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
                        assetId = viewModel.assetId,
                        priceAlertsCount = priceAlertsCount,
                        onPriceAlerts = onPriceAlerts,
                        onAddPriceAlertTarget = onAddPriceAlertTarget,
                    )
                }
                marketModel?.let {
                    assetMarket(it.currency, it.asset, it.marketInfo)
                    assetContract(it.asset, it.explorerName)
                    assetSupply(it.asset, it.marketInfo)
                    assetAllTime(it.currency, it.asset, it.marketInfo)
                    links(it.assetLinks)
                }
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
            data = { PropertyDataText(item.host.orEmpty(), badge = { DataBadgeChevron() }) },
            listPosition = ListPosition.getPosition(index, links.size)
        )
    }
}

private fun LazyListScope.assetContract(asset: Asset, explorerName: String) {
    val contract = contractMarketInfo(asset, explorerName) ?: return
    marketProperties(asset, listOf(contract))
}

private fun LazyListScope.assetMarket(currency: Currency, asset: Asset, marketInfo: AssetMarket?) {
    marketInfo ?: return
    val marketItems = buildMarketItems(marketInfo) { currency.compactFormatter(it) }

    marketProperties(asset, marketItems)
}

private fun LazyListScope.assetSupply(asset: Asset, marketInfo: AssetMarket?) {
    marketInfo ?: return
    val supplyItems = buildSupplyItems(
        marketInfo = marketInfo,
        compactSupplyFormatter = { asset.compactFormatter(it) },
        maxSupplyFormatter = { asset.formatSupply(it) },
    )

    marketProperties(asset, supplyItems)
}

private fun LazyListScope.assetAllTime(currency: Currency, asset: Asset, marketInfo: AssetMarket?) {
    marketInfo ?: return
    val allTime = listOfNotNull(
        marketInfo.allTimeHighValue?.let { AllTimeUIModel.High(it.date, it.value.toDouble(), it.percentage.toDouble()) },
        marketInfo.allTimeLowValue?.let { AllTimeUIModel.Low(it.date, it.value.toDouble(), it.percentage.toDouble()) },
    )

    allTimeProperties(asset, currency, allTime)
}

internal fun buildMarketItems(
    marketInfo: AssetMarket,
    compactCurrencyFormatter: (Double) -> String,
): List<MarketInfoUIModel> = listOfNotNull(
    marketInfo.marketCap?.let {
        MarketInfoUIModel(
            type = MarketInfoUIModel.MarketInfoTypeUIModel.MarketCap,
            value = compactCurrencyFormatter(it),
            badge = marketInfo.marketCapRank?.takeIf { rank -> rank in 1..1000 }?.let { "#$it" },
        )
    },
    marketInfo.marketCapFdv?.let {
        MarketInfoUIModel(
            type = MarketInfoUIModel.MarketInfoTypeUIModel.FDV,
            value = compactCurrencyFormatter(it),
            info = InfoSheetEntity.FullyDilutedValuation,
        )
    },
    marketInfo.totalVolume?.let {
        MarketInfoUIModel(
            type = MarketInfoUIModel.MarketInfoTypeUIModel.TradingVolume,
            value = compactCurrencyFormatter(it),
        )
    },
)

internal fun buildSupplyItems(
    marketInfo: AssetMarket,
    compactSupplyFormatter: (Double) -> String,
    maxSupplyFormatter: (Double) -> String,
): List<MarketInfoUIModel> = listOfNotNull(
    marketInfo.circulatingSupply?.let {
        MarketInfoUIModel(
            type = MarketInfoUIModel.MarketInfoTypeUIModel.CirculatingSupply,
            value = compactSupplyFormatter(it),
            info = InfoSheetEntity.CirculatingSupply,
        )
    },
    marketInfo.totalSupply?.let {
        MarketInfoUIModel(
            type = MarketInfoUIModel.MarketInfoTypeUIModel.TotalSupply,
            value = compactSupplyFormatter(it),
            info = InfoSheetEntity.TotalSupply,
        )
    },
    marketInfo.maxSupply?.let {
        MarketInfoUIModel(
            type = MarketInfoUIModel.MarketInfoTypeUIModel.MaxSupply,
            value = maxSupplyFormatter(it),
            info = InfoSheetEntity.MaxSupply,
        )
    },
)

internal fun contractMarketInfo(
    asset: Asset,
    explorerName: String,
    tokenExplorerUrl: (Asset, String, String) -> String? = ::defaultTokenExplorerUrl,
): MarketInfoUIModel? {
    val tokenId = asset.id.tokenId ?: return null
    return MarketInfoUIModel(
        type = MarketInfoUIModel.MarketInfoTypeUIModel.Contract,
        value = tokenId,
        explorerLink = tokenExplorerUrl(asset, explorerName, tokenId)
            ?.let { BlockExplorerLink(name = explorerName, link = it) },
    )
}

private fun defaultTokenExplorerUrl(asset: Asset, explorerName: String, tokenId: String): String? {
    return Explorer(asset.chain.string).getTokenUrl(explorerName, tokenId)
}

private fun LazyListScope.marketProperties(asset: Asset, items: List<MarketInfoUIModel>) {
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
                AddressPropertyItem(
                    title = R.string.asset_contract,
                    displayText = AddressFormatter(item.value, chain = asset.chain).value(),
                    copyValue = item.value,
                    explorerLink = item.explorerLink,
                    listPosition = position,
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
                    with(rowScope) { PropertyDataText(currency.format(item.value, dynamicPlace = true)) }
                    ListItemSupportText(item.percentage.formatAsPercentage(), color = item.percentage.toPriceState().color())
                }
            },
        )
    }

}
