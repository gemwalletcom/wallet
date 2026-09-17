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
import com.gemwallet.android.features.asset.viewmodels.chart.models.AllTimeUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartSectionUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.MarketInfoUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.MarketRowUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.AssetChartViewModel
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.ChartViewModel
import com.gemwallet.android.ui.LocalAddressService
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.AddressPropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.format.rememberFormattedAddress
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain

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
                            is ChartSectionUIModel.PriceAlerts -> item { LinkRow(section.model) { onPriceAlerts(viewModel.assetId) } }
                            is ChartSectionUIModel.SetPriceAlert -> item { LinkRow(section.model) { onAddPriceAlertTarget(viewModel.assetId) } }
                            is ChartSectionUIModel.Market -> marketRows(model.chain, section.rows)
                            is ChartSectionUIModel.Links -> links(section, uriHandler, context)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun LinkRow(model: ListItemModel, onClick: () -> Unit) {
    ListItem(
        model = model,
        listPosition = ListPosition.Single,
        modifier = Modifier
            .clickable(onClick = onClick)
            .testTag("assetChart"),
        accessory = { DataBadgeChevron() },
    )
}

private fun LazyListScope.links(section: ChartSectionUIModel.Links, uriHandler: UriHandler, context: Context) {
    if (section.links.isEmpty()) return
    item { SubheaderItem(section.title) }
    itemsPositioned(section.links) { position, link ->
        ListItem(
            model = link.model,
            listPosition = position,
            modifier = Modifier.clickable { uriHandler.open(context, link.url) },
            accessory = { DataBadgeChevron() },
        )
    }
}

private fun LazyListScope.marketRows(chain: Chain, items: List<MarketRowUIModel>) {
    itemsPositioned(items) { position, item ->
        when (item) {
            is MarketInfoUIModel -> MarketProperty(chain, item, position)
            is AllTimeUIModel -> ListItem(model = item.model, listPosition = position)
        }
    }
}

@Composable
private fun MarketProperty(chain: Chain, item: MarketInfoUIModel, position: ListPosition) {
    when (item.layout) {
        MarketInfoUIModel.Layout.Plain,
        MarketInfoUIModel.Layout.Badge -> ListItem(model = item.model, listPosition = position)
        MarketInfoUIModel.Layout.Address -> AddressPropertyItem(
            title = item.model.title,
            displayText = rememberFormattedAddress(item.model.subtitle.orEmpty(), chain),
            copyValue = item.model.subtitle.orEmpty(),
            explorerLink = item.explorerLink,
            listPosition = position,
        )
    }
}
