package com.gemwallet.android.ui.components.swap

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.dialog.DialogBarDismissType
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ProviderRowView
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.AssetRatePropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel
import com.gemwallet.android.ui.style.color
import com.gemwallet.android.ui.style.textStyle
import com.gemwallet.android.ui.theme.pendingColor
import uniffi.gemstone.GemProviderKind
import uniffi.gemstone.GemProviderRow
import uniffi.gemstone.GemSwapPriceImpactRow
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.SwapProvider

@Composable
fun SwapDetailsSummaryItem(model: SwapDetailsUIModel, onClick: () -> Unit, listPosition: ListPosition = ListPosition.Single) {
    val badgeText = model.summaryPriceImpactBadgeText

    ListItem(
        model = ListItemModel(title = stringResource(R.string.common_details), subtitle = model.rate.forward),
        listPosition = listPosition,
        modifier = Modifier.clickable(onClick = onClick),
        accessory = {
            if (badgeText != null) {
                DataBadgeChevron {
                    Text(
                        text = badgeText,
                        color = model.priceImpact.getColor(),
                        maxLines = 1,
                        overflow = TextOverflow.Clip,
                        softWrap = false,
                    )
                }
            } else {
                DataBadgeChevron()
            }
        },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SwapDetailsBottomSheet(
    isVisible: Boolean,
    isLoading: Boolean,
    model: SwapDetailsUIModel?,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
    expansion: SheetExpansion = SheetExpansion.Partial,
    showProviderSectionHeader: Boolean = false,
    onProviderSelect: ((SwapProvider) -> Unit)? = null,
) {
    ModalBottomSheet(
        item = model.takeIf { isVisible },
        onDismissRequest = onDismiss,
        modifier = modifier,
        expansion = expansion,
        title = { stringResource(R.string.common_details) },
        dismissType = DialogBarDismissType.Confirm,
    ) { model ->
        if (isLoading) {
            Box(modifier = Modifier.fillMaxWidth()) {
                CircularProgressIndicator20(modifier = Modifier.align(Alignment.Center))
            }
            return@ModalBottomSheet
        }

        LazyColumn {
            val providers = model.inlineProviders(onProviderSelect != null)
            val providerSectionTitle = when {
                onProviderSelect != null -> R.string.buy_providers_title
                showProviderSectionHeader -> R.string.common_provider
                else -> null
            }

            if (providerSectionTitle != null && providers.isNotEmpty()) {
                item {
                    SubheaderItem(providerSectionTitle)
                }
            }
            if (providers.size > 1 && onProviderSelect != null) {
                itemsIndexed(providers) { index, provider ->
                    ProviderRowView(
                        row = provider,
                        listPosition = ListPosition.getPosition(index, providers.size),
                        onClick = (provider.kind as? GemProviderKind.Swap)?.let { kind ->
                            {
                                onDismiss()
                                onProviderSelect(kind.provider)
                            }
                        },
                    )
                }
            } else {
                item {
                    ProviderRowView(row = providers.firstOrNull() ?: model.provider, listPosition = ListPosition.Single)
                }
            }
            item {
                AssetRatePropertyItem(stringResource(R.string.buy_rate), model.rate, ListPosition.First)
            }
            itemsIndexed(model.rows) { index, row ->
                GemListRowView(row = row, listPosition = ListPosition.getPosition(index + 1, model.rows.size + 1))
            }
        }
    }
}

private fun SwapDetailsUIModel.inlineProviders(isSelectionEnabled: Boolean): List<GemProviderRow> = when {
    isSelectionEnabled && isProviderSelectable -> providers
    else -> listOf(provider)
}

@Composable
private fun GemSwapPriceImpactRow?.getColor() = (this?.value?.tone ?: GemValueTone.NEUTRAL).color()
