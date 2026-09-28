package com.gemwallet.android.features.stake.presents.delegation

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.stake.viewmodels.delegation.DelegationViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_head.ValueListHead
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyAssetBalanceItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.wallet.core.primitives.ChainAddress

@Composable
fun DelegationScreen(onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction, onOpenAddress: (ChainAddress) -> Unit, onCancel: () -> Unit, viewModel: DelegationViewModel = hiltViewModel()) {
    val current by viewModel.details.collectAsStateWithLifecycle()
    val currentAssetInfo by viewModel.assetInfo.collectAsStateWithLifecycle()
    val details = current
    val asset = currentAssetInfo?.asset

    if (details == null || asset == null) {
        LoadingScene(title = stringResource(id = R.string.transfer_stake_title), onCancel = onCancel)
        return
    }
    val canClaimRewards = details.claim != null
    Scene(
        title = details.title.string(LocalContext.current),
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                ValueListHead(header = details.valueHeader)
            }
            val rewards = details.rewards
            val rowsCount = details.rows.size + if (rewards != null) 1 else 0
            itemsIndexed(details.rows) { index, row ->
                GemListRowView(
                    row = row,
                    listPosition = ListPosition.getPosition(index, rowsCount),
                    onSelectAddress = { onOpenAddress(ChainAddress(asset.id.chain, it)) },
                )
            }
            if (rewards != null) {
                item {
                    PropertyAssetBalanceItem(
                        asset = asset,
                        amount = rewards,
                        fiat = details.rewardsFiat,
                        title = stringResource(R.string.stake_rewards),
                        modifier = if (canClaimRewards) Modifier.clickable { viewModel.onClaimRewards(onConfirm) } else Modifier,
                        showChevron = canClaimRewards,
                        listPosition = ListPosition.getPosition(details.rows.size, rowsCount),
                    )
                }
            }

            if (details.actions.isNotEmpty()) {
                item { SubheaderItem(R.string.common_manage) }
            }
            itemsPositioned(details.actions) { position, item ->
                ListItem(
                    model = ListItemModel(title = stringResource(item.action.stringRes())),
                    listPosition = position,
                    modifier = Modifier.clickable { viewModel.onAction(item.destination, onAmount, onConfirm) },
                    accessory = { DataBadgeChevron() },
                )
            }
        }
    }
}
