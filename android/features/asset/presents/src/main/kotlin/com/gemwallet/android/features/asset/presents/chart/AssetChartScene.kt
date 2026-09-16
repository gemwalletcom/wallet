package com.gemwallet.android.features.asset.presents.chart

import android.content.Context
import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.UriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.features.asset.presents.localization.stringRes
import com.gemwallet.android.features.asset.viewmodels.chart.models.AllTimeUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartSectionUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.MarketInfoUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.MarketRowUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.AssetChartViewModel
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.ChartViewModel
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.LocalAddressService
import com.gemwallet.android.ui.R
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
import com.gemwallet.android.ui.components.list_item.property.SocialLinkUIModel
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.format.rememberFormattedAddress
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.smallIconSize
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
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
    val isChartRefreshing by chartViewModel.isRefreshing.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(
        message = toastMessage,
        iconRes = R.drawable.ic_notifications,
        onShown = onToastShown,
    )
    val uriHandler = LocalUriHandler.current
    val context = LocalContext.current

    Scene(
        title = title,
        onClose = onCancel,
        snackbar = snackbar,
    ) {
        PullToRefreshBox(
            isRefreshing = isChartRefreshing,
            onRefresh = {
                chartViewModel.refresh()
            },
            containerColor = PullToRefreshDefaults.indicatorContainerColor,
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                item { Chart(chartViewModel) }
                marketModel?.let { model ->
                    model.sections.forEach { section ->
                        when (section) {
                            is ChartSectionUIModel.PriceAlerts -> section.stringRes()?.let { title ->
                                item { PriceAlertsItem(title, section.count.toString()) { onPriceAlerts(viewModel.assetId) } }
                            }
                            ChartSectionUIModel.SetPriceAlert -> section.stringRes()?.let { title ->
                                item { PriceAlertsItem(title, "") { onAddPriceAlertTarget(viewModel.assetId) } }
                            }
                            is ChartSectionUIModel.Market -> marketRows(model.chain, model.currency, section.rows)
                            is ChartSectionUIModel.Links -> links(section.stringRes(), section.links, uriHandler, context)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun PriceAlertsItem(@StringRes title: Int, data: String, onClick: () -> Unit) {
    PropertyItem(
        modifier = Modifier
            .clickable(onClick = onClick)
            .testTag("assetChart"),
        title = { PropertyTitleText(title) },
        data = { PropertyDataText(text = data, badge = { DataBadgeChevron() }) },
        listPosition = ListPosition.Single,
    )
}

private fun LazyListScope.links(@StringRes title: Int?, links: List<SocialLinkUIModel>, uriHandler: UriHandler, context: Context) {
    if (links.isEmpty() || title == null) return
    item { SubheaderItem(title) }
    itemsIndexed(links) { index, item ->
        PropertyItem(
            modifier = Modifier.clickable { uriHandler.open(context, item.url) },
            title = { PropertyTitleText(item.label, trailing = { AsyncImage(model = item.icon, size = smallIconSize) }) },
            data = { PropertyDataText(item.host.orEmpty(), badge = { DataBadgeChevron() }) },
            listPosition = ListPosition.getPosition(index, links.size)
        )
    }
}

private fun LazyListScope.marketRows(chain: Chain, currency: Currency, items: List<MarketRowUIModel>) {
    itemsPositioned(items) { position, item ->
        when (item) {
            is MarketInfoUIModel -> MarketProperty(chain, item, position)
            is AllTimeUIModel -> AllTimeProperty(currency, item, position)
        }
    }
}

@Composable
private fun MarketProperty(chain: Chain, item: MarketInfoUIModel, position: ListPosition) {
    when (item.layout) {
        MarketInfoUIModel.Layout.Plain -> PropertyItem(item.label, item.value, listPosition = position, info = item.info)
        MarketInfoUIModel.Layout.Badge -> PropertyItem(
            title = {
                PropertyTitleText(
                    text = item.label,
                    badge = item.badge?.let { { ChipBadge(it) } },
                )
            },
            data = { PropertyDataText(item.value) },
            listPosition = position,
        )
        MarketInfoUIModel.Layout.Address -> AddressPropertyItem(
            title = item.label,
            displayText = rememberFormattedAddress(item.value, chain),
            copyValue = item.value,
            explorerLink = item.explorerLink,
            listPosition = position,
        )
    }
}

internal fun LazyListScope.allTimeProperties(currency: Currency, items: List<AllTimeUIModel>) {
    itemsPositioned(items) { position, item -> AllTimeProperty(currency, item, position) }
}

@Composable
private fun AllTimeProperty(currency: Currency, item: AllTimeUIModel, position: ListPosition) {
    val title = when (item) {
        is AllTimeUIModel.High -> R.string.asset_all_time_high
        is AllTimeUIModel.Low -> R.string.asset_all_time_low
    }
    ListItem(
        listPosition = position,
        title = { PropertyTitleText(text = stringResource(title)) },
        subtitle = { ListItemSupportText(DateFormat.getDateInstance(DateFormat.MEDIUM).format(Date(item.date))) },
        trailing = {
            val rowScope = this
            Column(horizontalAlignment = Alignment.End) {
                with(rowScope) { PropertyDataText(CurrencyFormatter(currency = currency).string(item.value)) }
                ListItemSupportText(item.percentage.formatAsPercentage(), color = item.percentage.toValueDirection().color())
            }
        },
    )
}
