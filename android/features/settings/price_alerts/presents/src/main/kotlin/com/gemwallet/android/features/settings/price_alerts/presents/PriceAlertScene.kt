package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material3.ExperimentalMaterial3Api
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults.Indicator
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ActionIcon
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.SwipeableItemWithActions
import com.gemwallet.android.ui.components.list_item.SwitchProperty
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.theme.headerIconSize
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingLarge
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.AssetId

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PriceAlertScene(
    assetInfo: AssetInfo? = null,
    data: Map<AssetId?, List<PriceAlertDataAggregate>>,
    enabled: Boolean,
    syncState: Boolean,
    isAssetView: Boolean,
    snackbar: SnackbarHostState? = null,
    onEnablePriceAlerts: (Boolean) -> Unit,
    onToggleAutoAlert: (Boolean) -> Unit,
    onAdd: () -> Unit,
    onAddTarget: (AssetId) -> Unit,
    onExclude: (Int) -> Unit,
    onChart: (AssetId) -> Unit,
    onRefresh: () -> Unit,
    onCancel: () -> Unit,
) {
    val reveable = remember { mutableStateOf<Int?>(null) }
    val pullToRefreshState = rememberPullToRefreshState()
    Scene(
        title = stringResource(R.string.settings_price_alerts_title),
        actions = @Composable {
            val assetId = assetInfo?.id()
            IconButton(onClick = if (assetId == null) onAdd else {
                { onAddTarget(assetId) }
            }) {
                Icon(imageVector = Icons.Default.Add, contentDescription = "")
            }
        },
        snackbar = snackbar,
        onClose = onCancel
    ) {
        PullToRefreshBox(
            modifier = Modifier.fillMaxSize(),
            isRefreshing = syncState,
            onRefresh = onRefresh,
            state = pullToRefreshState,
            indicator = {
                Indicator(
                    modifier = Modifier.align(Alignment.TopCenter),
                    isRefreshing = syncState,
                    state = pullToRefreshState,
                    containerColor = MaterialTheme.colorScheme.background
                )
            }
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                if (isAssetView) {
                    autoAlertToggle(assetInfo, data, onToggleAutoAlert)
                    val manualData = data.filterKeys { it != null }
                    emptyAlertingAssets(data.values.flatten().isEmpty())
                    assets(
                        reveable = reveable,
                        data = manualData,
                        isAssetView = isAssetView,
                        onChart = onChart,
                        onExclude = onExclude,
                    )
                } else {
                    item {
                        SwitchProperty(
                            text = stringResource(R.string.settings_enable_value, ""),
                            checked = enabled,
                            onCheckedChange = onEnablePriceAlerts
                        )
                        Text(
                            modifier = Modifier.padding(horizontal = paddingLarge),
                            text = stringResource(R.string.price_alerts_get_notified_explain_message),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.secondary,
                        )
                    }
                    emptyAlertingAssets(data.values.flatten().isEmpty())
                    assets(
                        reveable = reveable,
                        data = data,
                        isAssetView = isAssetView,
                        onChart = onChart,
                        onExclude = onExclude,
                    )
                }
            }
        }
    }
}

private fun LazyListScope.autoAlertToggle(
    assetInfo: AssetInfo?,
    data: Map<AssetId?, List<PriceAlertDataAggregate>>,
    onToggleAutoAlert: (Boolean) -> Unit,
) {
    item {
        val autoAlerts = data[null] ?: emptyList()
        val isAutoAlertEnabled = autoAlerts.isNotEmpty()
        val currentAssetInfo = assetInfo ?: return@item

        PriceAlertAutoAssetItem(
            assetInfo = currentAssetInfo,
            enabled = isAutoAlertEnabled,
            onCheckedChange = onToggleAutoAlert,
        )
        Text(
            modifier = Modifier.padding(horizontal = paddingLarge),
            text = stringResource(R.string.price_alerts_auto_footer),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.secondary,
        )
    }
}

private fun LazyListScope.emptyAlertingAssets(empty: Boolean) {
    if (!empty) {
        return
    }
    item {
        EmptyContentView(
            type = EmptyContentType.PriceAlerts,
            modifier = Modifier.fillParentMaxHeight(0.5f),
        )
    }
}

private fun LazyListScope.assets(
    reveable: MutableState<Int?>,
    data: Map<AssetId?, List<PriceAlertDataAggregate>>,
    isAssetView: Boolean,
    onChart: (AssetId) -> Unit,
    onExclude: (Int) -> Unit,
) {
    data.entries.forEach { item ->
        if (item.value.isEmpty()) return@forEach

        item.key?.let {
            item { SubheaderItem(if (isAssetView) stringResource(R.string.stake_active) else item.value.firstOrNull()?.title ?: "") }
        }
        assets(reveable, item.value, onChart.takeIf { !isAssetView }, onExclude)
    }
}

private fun LazyListScope.assets(
    reveable: MutableState<Int?>,
    data: List<PriceAlertDataAggregate>,
    onChart: ((AssetId) -> Unit)?,
    onExclude: (Int) -> Unit,
) {
    itemsPositioned(data/*, key = { _, item -> item.id}*/) { position, item ->
        var minActionWidth by remember { mutableStateOf(0.dp) }
        val density = LocalDensity.current

        SwipeableItemWithActions(
            isRevealed = reveable.value == item.id,
            actions = @Composable {
                ActionIcon(
                    modifier = Modifier
                        .widthIn(min = minActionWidth)
                        .heightIn(minActionWidth),
                    onClick = { onExclude(item.id) },
                    backgroundColor = MaterialTheme.colorScheme.error,
                    icon = Icons.Default.Delete,
                )
            },
            onExpanded = { reveable.value = item.id },
            onCollapsed = { reveable.value = null },
            listPosition = position,
        ) { position ->
            PriceAlertAssetItem(
                modifier = (onChart?.let { Modifier
                    .clickable(onClick = { onChart(item.assetId) }) } ?: Modifier)
                    .onSizeChanged {
                        minActionWidth = with(density) { it.height.toDp() }
                    },
                item = item,
                listPosition = position,
            )
        }
    }
}
