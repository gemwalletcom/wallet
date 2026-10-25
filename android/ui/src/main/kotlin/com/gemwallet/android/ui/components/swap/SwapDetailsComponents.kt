package com.gemwallet.android.ui.components.swap

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.text.TextAutoSize
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.sp
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
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
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
import uniffi.gemstone.GemProviderLoadingRow
import uniffi.gemstone.GemProviderRow
import uniffi.gemstone.GemSwapDetails
import uniffi.gemstone.GemSwapQuotesState
import uniffi.gemstone.SwapProvider

@Composable
fun SwapDetailsSummaryItem(details: GemSwapDetails, onClick: () -> Unit, listPosition: ListPosition = ListPosition.Single) {
    PropertyItem(
        modifier = Modifier.clickable(onClick = onClick),
        title = { PropertyTitleText(R.string.common_details) },
        data = {
            Text(
                text = details.summaryText(),
                modifier = Modifier.weight(1f, false),
                color = MaterialTheme.colorScheme.secondary,
                textAlign = TextAlign.End,
                maxLines = 1,
                style = MaterialTheme.typography.bodyLarge,
                autoSize = TextAutoSize.StepBased(
                    minFontSize = MaterialTheme.typography.bodySmall.fontSize,
                    maxFontSize = MaterialTheme.typography.bodyLarge.fontSize,
                    stepSize = 1.sp,
                ),
            )
            DataBadgeChevron()
        },
        listPosition = listPosition,
    )
}

@Composable
private fun GemSwapDetails.summaryText(): AnnotatedString = buildAnnotatedString {
    append(rate?.forward.orEmpty())
    summary.priceImpactRow?.takeIf { it.showsInSummary }?.value?.let { priceImpact ->
        append(" (")
        withStyle(SpanStyle(color = priceImpact.tone.color())) { append(priceImpact.text()) }
        append(")")
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
