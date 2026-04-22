package com.gemwallet.android.ui.components.swap

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.SwapVert
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.SelectionCheckmark
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel
import com.gemwallet.android.ui.models.swap.SwapPriceImpactUIModel
import com.gemwallet.android.ui.models.swap.SwapProviderUIModel
import com.gemwallet.android.ui.models.swap.SwapRateUIModel
import com.wallet.core.primitives.swap.SwapPriceImpactType
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.listItemIconSize
import uniffi.gemstone.SwapperProvider

@Composable
fun SwapDetailsSummaryItem(
    model: SwapDetailsUIModel,
    onClick: () -> Unit,
) {
    val badgeText = model.summaryPriceImpactBadgeText

    PropertyItem(
        modifier = Modifier.clickable(onClick = onClick),
        title = { PropertyTitleText(R.string.common_details) },
        data = {
            PropertyDataText(
                text = model.rate.forward,
                badge = if (badgeText != null) {
                    {
                        DataBadgeChevron {
                            Text(
                                text = badgeText,
                                color = model.priceImpact.getColor(),
                                maxLines = 1,
                                overflow = TextOverflow.Clip,
                                softWrap = false,
                            )
                        }
                    }
                } else {
                    { DataBadgeChevron() }
                },
            )
        },
        listPosition = ListPosition.Single,
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
    skipPartiallyExpanded: Boolean = false,
    showProviderSectionHeader: Boolean = false,
    onProviderSelect: ((SwapperProvider) -> Unit)? = null,
) {
    if (model == null) return

    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismiss,
        modifier = modifier,
        skipPartiallyExpanded = skipPartiallyExpanded,
        title = stringResource(R.string.common_details),
    ) {
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
            item {
                SwapRatePropertyItem(model.rate, ListPosition.First)
            }
            model.estimatedTime?.let {
                item {
                    PropertyItem(
                        title = R.string.swap_estimated_time_title,
                        data = it,
                        listPosition = ListPosition.Middle,
                    )
                }
            }
            model.priceImpact?.let {
                item {
                    PriceImpactPropertyItem(it, ListPosition.Middle)
                }
            }
            item {
                PropertyItem(
                    title = R.string.swap_min_receive,
                    data = model.minimumReceive,
                    listPosition = ListPosition.Middle,
                )
            }
            item {
                PropertyItem(
                    title = R.string.swap_slippage,
                    data = model.slippageText,
                    info = InfoSheetEntity.Slippage,
                    listPosition = ListPosition.Last,
                )
            }
        }
    }
}

@Composable
private fun SwapProviderListItemView(
    provider: SwapProviderUIModel,
    listPosition: ListPosition,
    isSelected: Boolean,
    onProviderSelect: (SwapperProvider) -> Unit,
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
private fun SwapRatePropertyItem(rate: SwapRateUIModel, listPosition: ListPosition) {
    var showReverse by remember { mutableStateOf(false) }
    val displayedRate = if (showReverse) rate.reverse else rate.forward

    PropertyItem(
        modifier = Modifier.clickable { showReverse = !showReverse },
        title = { PropertyTitleText(R.string.buy_rate) },
        data = {
            PropertyDataText(
                text = displayedRate,
                badge = {
                    Spacer4()
                    Icon(
                        modifier = Modifier.clip(MaterialTheme.shapes.small),
                        imageVector = Icons.Default.SwapVert,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.secondary,
                    )
                },
            )
        },
        listPosition = listPosition,
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
private fun SwapProviderIcon(icon: Any, size: Dp) {
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
private fun PriceImpactPropertyItem(
    priceImpact: SwapPriceImpactUIModel,
    listPosition: ListPosition,
) {
    PropertyItem(
        title = R.string.swap_price_impact,
        info = InfoSheetEntity.PriceImpactInfo,
        data = priceImpact.displayText,
        dataColor = priceImpact.getColor(),
        listPosition = listPosition,
    )
}

@Composable
private fun SwapPriceImpactUIModel?.getColor() = when (this?.type) {
    SwapPriceImpactType.Positive -> MaterialTheme.colorScheme.tertiary
    SwapPriceImpactType.Medium -> pendingColor
    SwapPriceImpactType.High -> MaterialTheme.colorScheme.error
    SwapPriceImpactType.Low,
    null -> MaterialTheme.colorScheme.secondary
}
