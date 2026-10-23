package com.gemwallet.android.ui.components.swap

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
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
import com.gemwallet.android.ui.components.list_item.ProviderLoadingRowView
import com.gemwallet.android.ui.components.list_item.ProviderRowView
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.WarningItem
import com.gemwallet.android.ui.components.list_item.property.AssetRatePropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.color
import com.gemwallet.android.ui.style.textStyle
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.space4
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemProviderKind
import uniffi.gemstone.GemProviderLoadingRow
import uniffi.gemstone.GemProviderRow
import uniffi.gemstone.GemSwapDetails
import uniffi.gemstone.GemSwapQuotesState
import uniffi.gemstone.SwapProvider

@Composable
fun SwapDetailsSummaryItem(details: GemSwapDetails?, onClick: () -> Unit, progress: String? = null, listPosition: ListPosition = ListPosition.Single) {
    val priceImpact = details?.summary?.priceImpactRow?.takeIf { it.showsInSummary }?.value

    PropertyItem(
        modifier = Modifier.clickable(onClick = onClick),
        title = { PropertyTitleText(R.string.common_details) },
        data = {
            QuoteProgress(progress)
            PropertyDataText(
                text = details?.rate?.forward.orEmpty(),
                badge = {
                    DataBadgeChevron(
                        content = priceImpact?.let {
                            {
                                Text(
                                    text = "(${it.text()})",
                                    color = it.tone.color(),
                                    maxLines = 1,
                                    overflow = TextOverflow.Clip,
                                    softWrap = false,
                                )
                            }
                        },
                    )
                },
            )
        },
        listPosition = listPosition,
    )
}

@Composable
private fun QuoteProgress(progress: String?) {
    AnimatedContent(
        targetState = progress,
        contentKey = { it != null },
        transitionSpec = { fadeIn() togetherWith fadeOut() },
        label = "quote_progress",
    ) { text ->
        text?.let {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(space4)) {
                CircularProgressIndicator16()
                Text(text = it, style = MaterialTheme.typography.labelMedium, color = MaterialTheme.colorScheme.secondary)
                Spacer4()
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SwapDetailsBottomSheet(
    isVisible: Boolean,
    details: GemSwapDetails?,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
    quotesState: GemSwapQuotesState = GemSwapQuotesState.Quotes,
    expansion: SheetExpansion = SheetExpansion.Partial,
    providers: List<GemProviderRow> = emptyList(),
    pendingProviders: List<GemProviderLoadingRow> = emptyList(),
    isProviderSelectable: Boolean = false,
    onProviderSelect: ((SwapProvider) -> Unit)? = null,
) {
    val context = LocalContext.current
    ModalBottomSheet(
        item = (quotesState to details).takeIf { isVisible },
        onDismissRequest = onDismiss,
        modifier = modifier,
        expansion = expansion,
        title = { stringResource(if (details == null) R.string.buy_providers_title else R.string.common_details) },
        dismissType = DialogBarDismissType.Confirm,
    ) { (quotesState, details) ->
        when (quotesState) {
            GemSwapQuotesState.Loading -> Box(modifier = Modifier.fillMaxWidth()) {
                CircularProgressIndicator20(modifier = Modifier.align(Alignment.Center))
            }

            is GemSwapQuotesState.Failed -> WarningItem(
                title = stringResource(R.string.errors_error_occurred),
                message = quotesState.error.text(context),
                color = MaterialTheme.colorScheme.error,
                position = ListPosition.Single,
            )

            GemSwapQuotesState.Empty -> WarningItem(
                title = stringResource(R.string.errors_error_occurred),
                color = MaterialTheme.colorScheme.error,
                position = ListPosition.Single,
            )

            GemSwapQuotesState.Quotes -> SwapDetailsList(
                details = details,
                providers = providers,
                pendingProviders = pendingProviders,
                isProviderSelectable = isProviderSelectable,
                onProviderSelect = onProviderSelect,
                onDismiss = onDismiss,
            )
        }
    }
}

@Composable
private fun SwapDetailsList(details: GemSwapDetails?, providers: List<GemProviderRow>, pendingProviders: List<GemProviderLoadingRow>, isProviderSelectable: Boolean, onProviderSelect: ((SwapProvider) -> Unit)?, onDismiss: () -> Unit) {
    val context = LocalContext.current
    LazyColumn {
        item {
            SubheaderItem(GemListRowTitle.PROVIDER.text(context))
        }
        if (onProviderSelect != null && isProviderSelectable) {
            val rows = providers.size + pendingProviders.size
            itemsIndexed(providers) { index, provider ->
                ProviderRowView(
                    row = provider,
                    listPosition = ListPosition.getPosition(index, rows),
                    onClick = (provider.kind as? GemProviderKind.Swap)?.let { kind ->
                        {
                            onDismiss()
                            onProviderSelect(kind.provider)
                        }
                    },
                )
            }
            itemsIndexed(pendingProviders) { index, provider ->
                ProviderLoadingRowView(row = provider, listPosition = ListPosition.getPosition(providers.size + index, rows))
            }
        } else if (details != null) {
            item {
                ProviderRowView(row = details.provider, listPosition = ListPosition.Single)
            }
        }
        if (details == null) {
            return@LazyColumn
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

private val GemSwapDetails.rate: AssetRatePair?
    get() = summary.rate?.let(AssetRateFormatter()::format)
