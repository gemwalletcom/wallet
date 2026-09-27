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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.domains.swap.AssetRateFormatter
import com.gemwallet.android.domains.swap.AssetRatePair
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
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.color
import com.gemwallet.android.ui.style.textStyle
import com.gemwallet.android.ui.theme.pendingColor
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemProviderKind
import uniffi.gemstone.GemProviderRow
import uniffi.gemstone.GemSwapDetails
import uniffi.gemstone.SwapProvider

@Composable
fun SwapDetailsSummaryItem(details: GemSwapDetails, onClick: () -> Unit, listPosition: ListPosition = ListPosition.Single) {
    val priceImpact = details.summary.priceImpactRow?.takeIf { it.showsInSummary }?.value

    ListItem(
        model = ListItemModel(title = stringResource(R.string.common_details), subtitle = details.rate?.forward),
        listPosition = listPosition,
        modifier = Modifier.clickable(onClick = onClick),
        accessory = {
            if (priceImpact != null) {
                DataBadgeChevron {
                    Text(
                        text = "(${priceImpact.text()})",
                        color = priceImpact.tone.color(),
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
    details: GemSwapDetails?,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
    expansion: SheetExpansion = SheetExpansion.Partial,
    providers: List<GemProviderRow> = emptyList(),
    isProviderSelectable: Boolean = false,
    onProviderSelect: ((SwapProvider) -> Unit)? = null,
) {
    val context = LocalContext.current
    ModalBottomSheet(
        item = details.takeIf { isVisible },
        onDismissRequest = onDismiss,
        modifier = modifier,
        expansion = expansion,
        title = { stringResource(R.string.common_details) },
        dismissType = DialogBarDismissType.Confirm,
    ) { details ->
        if (isLoading) {
            Box(modifier = Modifier.fillMaxWidth()) {
                CircularProgressIndicator20(modifier = Modifier.align(Alignment.Center))
            }
            return@ModalBottomSheet
        }

        LazyColumn {
            item {
                SubheaderItem(GemListRowTitle.PROVIDER.text(context))
            }
            if (onProviderSelect != null && isProviderSelectable) {
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
                    ProviderRowView(row = details.provider, listPosition = ListPosition.Single)
                }
            }
            val rate = details.rate
            val rateRows = if (rate != null) 1 else 0
            rate?.let {
                item {
                    AssetRatePropertyItem(stringResource(R.string.buy_rate), it, ListPosition.First)
                }
            }
            itemsIndexed(details.rows) { index, row ->
                GemListRowView(row = row, listPosition = ListPosition.getPosition(index + rateRows, details.rows.size + rateRows))
            }
        }
    }
}

private val GemSwapDetails.rate: AssetRatePair?
    get() = summary.rate?.let(AssetRateFormatter()::format)
