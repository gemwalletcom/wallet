package com.gemwallet.android.features.price_alerts.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.ActionIcon
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.SwipeableItemWithActions
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.theme.paddingLarge
import com.gemwallet.android.ui.theme.space0
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetPriceAlerts
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemPriceAlertItem
import uniffi.gemstone.GemPriceAlertToggle
import uniffi.gemstone.priceAlertsToggleRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun PriceAlertsScene(
    sections: List<ListSection<GemPriceAlertItem>>,
    phase: GemListPhase?,
    syncState: Boolean,
    snackbar: SnackbarHostState,
    error: String?,
    onErrorShown: () -> Unit,
    header: LazyListScope.() -> Unit,
    onChart: ((AssetId) -> Unit)?,
    onAction: (PriceAlertsAction) -> Unit,
) {
    val revealable = remember { mutableStateOf<String?>(null) }
    LaunchedEffect(error) {
        error?.let {
            snackbar.showSnackbar(it, R.drawable.ic_error)
            onErrorShown()
        }
    }
    Scene(
        title = stringResource(R.string.settings_price_alerts_title),
        actions = @Composable {
            IconButton(onClick = { onAction(PriceAlertsAction.Add) }) {
                Icon(imageVector = AppIcons.Add, contentDescription = "")
            }
        },
        snackbar = snackbar,
        onClose = { onAction(PriceAlertsAction.Close) },
    ) {
        PullToRefreshBox(
            modifier = Modifier.fillMaxSize(),
            isRefreshing = syncState,
            onRefresh = { onAction(PriceAlertsAction.Refresh) },
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                (phase as? GemListPhase.Error)?.let { item { GemListRowView(row = GemListRow.Error(it.error), listPosition = ListPosition.Single) } }
                header()
                emptyAlertingAssets(phase)
                assets(
                    revealable = revealable,
                    sections = sections,
                    onChart = onChart,
                    onExclude = { onAction(PriceAlertsAction.Exclude(it)) },
                )
            }
        }
    }
}

internal fun LazyListScope.priceAlertsToggle(enabled: Boolean, onToggle: (Boolean) -> Unit) {
    item {
        GemListRowView(
            row = priceAlertsToggleRow(enabled),
            listPosition = ListPosition.Single,
            onToggle = { _, isOn -> onToggle(isOn) },
        )
        Text(
            modifier = Modifier.padding(horizontal = paddingLarge),
            text = stringResource(R.string.price_alerts_get_notified_explain_message),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.secondary,
        )
    }
}

internal fun LazyListScope.autoAlertToggle(assetAlerts: GemAssetPriceAlerts, onToggle: (Boolean) -> Unit) {
    item {
        PriceAlertAutoAssetItem(
            row = assetAlerts.autoRow.row,
            enabled = assetAlerts.autoAlert == GemPriceAlertToggle.ENABLED,
            onCheckedChange = onToggle,
        )
        Text(
            modifier = Modifier.padding(horizontal = paddingLarge),
            text = stringResource(R.string.price_alerts_auto_footer),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.secondary,
        )
    }
}

private fun LazyListScope.emptyAlertingAssets(phase: GemListPhase?) {
    if (phase !is GemListPhase.Empty) {
        return
    }
    item {
        EmptyContentView(
            state = phase.state,
            modifier = Modifier.fillParentMaxHeight(0.5f),
        )
    }
}

private fun LazyListScope.assets(revealable: MutableState<String?>, sections: List<ListSection<GemPriceAlertItem>>, onChart: ((AssetId) -> Unit)?, onExclude: (String) -> Unit) {
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
            PriceAlertItem(
                modifier = (
                    onChart?.let {
                        Modifier
                            .clickable(onClick = { onChart(item.data.asset.toPrimitives().id) })
                    } ?: Modifier
                    )
                    .onSizeChanged {
                        minActionWidth = with(density) { it.height.toDp() }
                    }
                    .testTag(item.data.asset.toPrimitives().id.toIdentifier()),
                item = item,
                listPosition = position,
            )
        }
    }
}
