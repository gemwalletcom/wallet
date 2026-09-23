package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetDetailsAction
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open

@Composable
internal fun AssetDetailRowItem(uiState: AssetInfoUIModel, row: AssetInfoUIModel.RowUIModel, listPosition: ListPosition, onAction: (AssetDetailsAction) -> Unit) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    when (row) {
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

        is AssetInfoUIModel.RowUIModel.Row -> GemListRowView(
            row = row.row,
            listPosition = listPosition,
            onSelect = row.action?.let { action -> { onAction(action) } },
            modifier = if (row.action is AssetDetailsAction.OpenChart) Modifier.testTag("assetChart") else Modifier,
        )
    }
}
