package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.features.settings.price_alerts.viewmodels.PriceAlertItemUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.ActionIcon
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.SwipeableItemWithActions
import com.gemwallet.android.ui.components.list_item.SwitchProperty
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.theme.headerIconSize
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingLarge
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space0
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetPriceAlerts
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemPriceAlertToggle

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun PriceAlertScene(
    asset: AssetInfoDataAggregate? = null,
    sections: List<ListSection<PriceAlertItemUIModel>>,
    errorRow: GemListRow?,
    assetAlerts: GemAssetPriceAlerts?,
    showsEmpty: Boolean,
    enabled: Boolean,
    syncState: Boolean,
    isAssetView: Boolean,
    snackbar: SnackbarHostState? = null,
    onAction: (PriceAlertAction) -> Unit,
) {
    val revealable = remember { mutableStateOf<String?>(null) }
    Scene(
        title = stringResource(R.string.settings_price_alerts_title),
        actions = @Composable {
            val assetId = asset?.id
            IconButton(
                onClick = if (assetId == null) {
                    { onAction(PriceAlertAction.Add) }
                } else {
                    { onAction(PriceAlertAction.AddTarget(assetId)) }
                },
            ) {
                Icon(imageVector = AppIcons.Add, contentDescription = "")
            }
        },
        snackbar = snackbar,
        onClose = { onAction(PriceAlertAction.Close) },
    ) {
        PullToRefreshBox(
            modifier = Modifier.fillMaxSize(),
            isRefreshing = syncState,
            onRefresh = { onAction(PriceAlertAction.Refresh) },
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                errorRow?.let { row -> item { GemListRowView(row = row, listPosition = ListPosition.Single) } }
                if (isAssetView) {
                    autoAlertToggle(asset, assetAlerts) { onAction(PriceAlertAction.ToggleAutoAlert(it)) }
                    emptyAlertingAssets(showsEmpty)
                    assets(
                        revealable = revealable,
                        sections = sections,
                        onChart = null,
                        onExclude = { onAction(PriceAlertAction.Exclude(it)) },
                    )
                } else {
                    item {
                        SwitchProperty(
                            text = stringResource(R.string.settings_enable_value, stringResource(R.string.settings_price_alerts_title)),
                            checked = enabled,
                            onCheckedChange = { onAction(PriceAlertAction.TogglePriceAlerts(it)) },
                        )
                        Text(
                            modifier = Modifier.padding(horizontal = paddingLarge),
                            text = stringResource(R.string.price_alerts_get_notified_explain_message),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.secondary,
                        )
                    }
                    emptyAlertingAssets(showsEmpty)
                    assets(
                        revealable = revealable,
                        sections = sections,
                        onChart = { onAction(PriceAlertAction.OpenChart(it)) },
                        onExclude = { onAction(PriceAlertAction.Exclude(it)) },
                    )
                }
            }
        }
    }
}

private fun LazyListScope.autoAlertToggle(asset: AssetInfoDataAggregate?, assetAlerts: GemAssetPriceAlerts?, onToggleAutoAlert: (Boolean) -> Unit) {
    item {
        val currentAsset = asset ?: return@item
        val alerts = assetAlerts ?: return@item

        PriceAlertAutoAssetItem(
            asset = currentAsset,
            row = alerts.autoRow,
            enabled = alerts.autoAlert == GemPriceAlertToggle.ENABLED,
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
            type = EmptyContentType(GemEmptyStateKind.PRICE_ALERTS),
            modifier = Modifier.fillParentMaxHeight(0.5f),
        )
    }
}

private fun LazyListScope.assets(revealable: MutableState<String?>, sections: List<ListSection<PriceAlertItemUIModel>>, onChart: ((AssetId) -> Unit)?, onExclude: (String) -> Unit) {
    listSections(sections) { position, item ->
        var minActionWidth by remember { mutableStateOf(space0) }
        val density = LocalDensity.current

        SwipeableItemWithActions(
            isRevealed = revealable.value == item.id,
            actions = @Composable {
                ActionIcon(
                    modifier = Modifier
                        .widthIn(min = minActionWidth)
                        .heightIn(minActionWidth),
                    onClick = { onExclude(item.id) },
                    backgroundColor = MaterialTheme.colorScheme.error,
                    icon = AppIcons.Delete,
                )
            },
            onExpanded = { revealable.value = item.id },
            onCollapsed = { revealable.value = null },
            listPosition = position,
        ) { position ->
            PriceAlertAssetItem(
                modifier = (
                    onChart?.let {
                        Modifier
                            .clickable(onClick = { onChart(item.asset.id) })
                    } ?: Modifier
                    )
                    .onSizeChanged {
                        minActionWidth = with(density) { it.height.toDp() }
                    },
                item = item,
                listPosition = position,
            )
        }
    }
}
