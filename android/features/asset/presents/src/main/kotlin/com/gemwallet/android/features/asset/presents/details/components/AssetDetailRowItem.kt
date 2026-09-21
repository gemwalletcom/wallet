package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.asset.presents.details.AssetDetailsAction
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyNetworkItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import uniffi.gemstone.GemAssetNetworkDestination
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle

@Composable
internal fun AssetDetailRowItem(uiState: AssetInfoUIModel, row: AssetInfoUIModel.RowUIModel, listPosition: ListPosition, onSelect: (GemListRowTitle) -> Unit, onAction: (AssetDetailsAction) -> Unit) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    when (row) {
        AssetInfoUIModel.RowUIModel.Price -> PriceItem(uiState, listPosition, onChart = { onAction(AssetDetailsAction.OpenChart(it)) })

        is AssetInfoUIModel.RowUIModel.Network -> PropertyNetworkItem(
            chain = uiState.asset.chain,
            value = row.name,
            listPosition = listPosition,
            onOpenNetwork = uiState.networkNavigation?.let { { onAction(it) } },
        )

        is AssetInfoUIModel.RowUIModel.Balance -> {
            val onBalance: (() -> Unit)? = when (row.type) {
                AssetInfoUIModel.BalanceViewType.Available,
                AssetInfoUIModel.BalanceViewType.PendingUnconfirmed,
                -> null

                AssetInfoUIModel.BalanceViewType.Stake -> {
                    { onAction(AssetDetailsAction.Stake(uiState.asset.id)) }
                }

                AssetInfoUIModel.BalanceViewType.Earn -> {
                    { onAction(AssetDetailsAction.Earn(uiState.asset.id)) }
                }

                AssetInfoUIModel.BalanceViewType.Reserved -> row.url?.let { url ->
                    { uriHandler.open(context, url) }
                }
            }
            ListItem(
                model = row.model,
                listPosition = listPosition,
                modifier = onBalance?.let { Modifier.clickable(onClick = it).testTag("assetStake") } ?: Modifier,
                accessory = onBalance?.let { { DataBadgeChevron() } },
            )
        }

        is AssetInfoUIModel.RowUIModel.Earn -> GemListRowView(
            row = row.row,
            listPosition = listPosition,
            modifier = Modifier.clickable { onAction(AssetDetailsAction.Earn(uiState.asset.id)) },
            accessory = { DataBadgeChevron() },
        )

        is AssetInfoUIModel.RowUIModel.Row -> when (val listRow = row.row) {
            is GemListRow.Link -> GemListRowView(row = listRow, listPosition = listPosition, modifier = Modifier.clickable { onSelect(listRow.title) })
            else -> GemListRowView(row = listRow, listPosition = listPosition)
        }
    }
}

private val AssetInfoUIModel.networkNavigation: AssetDetailsAction.Navigation?
    get() = when (val destination = networkDestination) {
        is GemAssetNetworkDestination.Asset -> AssetDetailsAction.OpenNetwork(destination.asset.toPrimitives().id)
        is GemAssetNetworkDestination.Assets -> AssetDetailsAction.OpenNetworkAssets(destination.chain.requireChain())
        null -> null
    }
