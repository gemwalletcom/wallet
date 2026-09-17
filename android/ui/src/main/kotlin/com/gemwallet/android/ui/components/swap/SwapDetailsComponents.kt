package com.gemwallet.android.ui.components.swap

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
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
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.domains.duration.formatEstimatedConfirmation
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.dialog.DialogBarDismissType
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.AssetRatePropertyItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.swap.SwapDetailRowUIModel
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel
import com.gemwallet.android.ui.models.swap.SwapPriceImpactUIModel
import com.gemwallet.android.ui.models.swap.SwapProviderUIModel
import com.gemwallet.android.ui.style.textStyle
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.pendingColor
import uniffi.gemstone.SwapPriceImpactType
import uniffi.gemstone.SwapProvider

@Composable
fun SwapDetailsSummaryItem(
    model: SwapDetailsUIModel,
    onClick: () -> Unit,
    listPosition: ListPosition = ListPosition.Single,
) {
    val badgeText = model.summaryPriceImpactBadgeText

    ListItem(
        model = ListItemModel(title = stringResource(R.string.common_details), subtitle = model.rate.forward),
        listPosition = listPosition,
        modifier = Modifier.clickable(onClick = onClick),
        accessory = {
            if (badgeText != null) {
                Text(
                    text = badgeText,
                    color = model.priceImpact.getColor(),
                    maxLines = 1,
                    overflow = TextOverflow.Clip,
                    softWrap = false,
                )
            }
            DataBadgeChevron()
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
                    SwapProviderListItemView(
                        provider = provider,
                        listPosition = ListPosition.getPosition(index, providers.size),
                        isSelected = provider.id == model.provider.id,
                        onProviderSelect = { selected ->
                            onDismiss()
                            onProviderSelect(selected)
                        },
                    )
                }
            } else {
                item {
                    SwapCurrentProviderRow(
                        provider = providers.firstOrNull() ?: model.provider,
                    )
                }
            }
            itemsIndexed(model.rows) { index, row ->
                val listPosition = ListPosition.getPosition(index, model.rows.size)
                when (row) {
                    is SwapDetailRowUIModel.Rate -> AssetRatePropertyItem(row.rate, listPosition)
                    is SwapDetailRowUIModel.EstimatedTime -> formatEstimatedConfirmation(row.seconds).takeIf { it.isNotEmpty() }?.let {
                        ListItem(model = ListItemModel(title = stringResource(R.string.swap_estimated_time_title), subtitle = it), listPosition = listPosition)
                    }
                    is SwapDetailRowUIModel.PriceImpact -> ListItem(
                        model = ListItemModel(
                            title = stringResource(R.string.swap_price_impact),
                            subtitle = row.model.displayText,
                            subtitleStyle = row.model.type.textStyle(),
                            info = InfoSheetEntity.PriceImpactInfo,
                        ),
                        listPosition = listPosition,
                    )
                    is SwapDetailRowUIModel.MinimumReceive -> ListItem(model = ListItemModel(title = stringResource(R.string.swap_min_receive), subtitle = row.text), listPosition = listPosition)
                    is SwapDetailRowUIModel.Slippage -> ListItem(
                        model = ListItemModel(
                            title = stringResource(R.string.swap_slippage),
                            subtitle = row.text ?: stringResource(R.string.swap_slippage_auto),
                            info = InfoSheetEntity.Slippage,
                        ),
                        listPosition = listPosition,
                    )
                }
            }
        }
    }
}

@Composable
private fun SwapProviderListItemView(
    provider: SwapProviderUIModel,
    listPosition: ListPosition,
    isSelected: Boolean,
    onProviderSelect: (SwapProvider) -> Unit,
) {
    ListItem(
        modifier = Modifier.clickable { onProviderSelect(provider.id) },
        leading = {
            if (isSelected) {
                IconWithBadge(
                    icon = provider.icon,
                    size = listItemIconSize,
                    badge = { SelectionCheckmark() },
                )
            } else {
                SwapProviderIcon(provider.icon, listItemIconSize)
            }
        },
        title = { ListItemTitleText(provider.title) },
        trailing = { SwapProviderAmounts(provider) },
        listPosition = listPosition,
    )
}

@Composable
private fun SwapCurrentProviderRow(
    provider: SwapProviderUIModel,
) {
    ListItem(
        leading = { SwapProviderIcon(provider.icon, listItemIconSize) },
        title = {
            ListItemTitleText(provider.title)
        },
        trailing = {
            SwapProviderAmounts(provider)
        },
        listPosition = ListPosition.Single,
    )
}

@Composable
private fun SwapProviderAmounts(provider: SwapProviderUIModel) {
    Column(horizontalAlignment = Alignment.End) {
        provider.amount?.let {
            ListItemTitleText(it)
        }
        provider.fiat?.let {
            ListItemSupportText(it)
        }
    }
}

@Composable
private fun SwapProviderIcon(icon: Any?, size: Dp) {
    AsyncImage(model = icon, size = size)
}

private fun SwapDetailsUIModel.inlineProviders(isSelectionEnabled: Boolean): List<SwapProviderUIModel> {
    if (!isSelectionEnabled || !isProviderSelectable) {
        return listOf(provider)
    }

    val topProviders = providers.take(MAX_INLINE_PROVIDERS).toMutableList()
    if (topProviders.none { it.id == provider.id }) {
        topProviders.add(0, provider)
    }

    return topProviders
        .distinctBy { it.id }
        .take(MAX_INLINE_PROVIDERS)
}

private const val MAX_INLINE_PROVIDERS = 3

@Composable
private fun SwapPriceImpactUIModel?.getColor() = when (this?.type) {
    SwapPriceImpactType.POSITIVE -> MaterialTheme.colorScheme.tertiary
    SwapPriceImpactType.MEDIUM -> pendingColor
    SwapPriceImpactType.HIGH -> MaterialTheme.colorScheme.error
    SwapPriceImpactType.LOW,
    null -> MaterialTheme.colorScheme.secondary
}
