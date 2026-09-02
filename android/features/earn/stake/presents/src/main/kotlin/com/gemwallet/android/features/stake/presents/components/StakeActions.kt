package com.gemwallet.android.features.stake.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Resource
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.theme.secondaryFaded
import uniffi.gemstone.GemStakeAction
import uniffi.gemstone.GemStakeActionItem
import com.wallet.core.primitives.AssetId

internal fun LazyListScope.stakeActions(
    actions: List<GemStakeActionItem>,
    rewardsText: String,
    assetId: AssetId,
    amountAction: AmountTransactionAction,
    onRewards: () -> Unit
) {
    if (actions.isEmpty()) {
        return
    }
    item {
        SubheaderItem(R.string.common_manage)
    }
    itemsPositioned(actions) { position, item ->
        val action = item.action
        val title = when (action) {
            GemStakeAction.CLAIM_REWARDS -> R.string.transfer_claim_rewards_title
            GemStakeAction.STAKE -> R.string.transfer_stake_title
            GemStakeAction.FREEZE -> R.string.transfer_freeze_title
            GemStakeAction.UNFREEZE -> R.string.transfer_unfreeze_title
        }
        val onClick = when (action) {
            GemStakeAction.STAKE -> {
                { amountAction(AmountParams.Stake.Delegate(assetId)) }
            }
            GemStakeAction.FREEZE -> {
                { amountAction(AmountParams.Stake.Freeze(assetId, Resource.Bandwidth)) }
            }
            GemStakeAction.UNFREEZE -> {
                { amountAction(AmountParams.Stake.Unfreeze(assetId, Resource.Bandwidth)) }
            }
            GemStakeAction.CLAIM_REWARDS -> onRewards
        }
        val info = InfoSheetEntity.StakeFrozenRequired(assetId.getIconUrl()).takeIf { item.requiresFrozenBalance }
        var showInfo by remember { mutableStateOf(false) }
        PropertyItem(
            modifier = Modifier.clickable(enabled = item.isEnabled) {
                if (item.requiresFrozenBalance) showInfo = true else onClick()
            },
            title = {
                PropertyTitleText(
                    text = title,
                    color = if (item.requiresFrozenBalance) MaterialTheme.colorScheme.secondaryFaded else MaterialTheme.colorScheme.onSurface,
                    info = info,
                )
            },
            data = {
                PropertyDataText(
                    text = if (action == GemStakeAction.CLAIM_REWARDS) rewardsText else "",
                    badge = { DataBadgeChevron() },
                )
            },
            listPosition = position
        )
        info?.let { if (showInfo) InfoBottomSheet(it) { showInfo = false } }
    }
}
