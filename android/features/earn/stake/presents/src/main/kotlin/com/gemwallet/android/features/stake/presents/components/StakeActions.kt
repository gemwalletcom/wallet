package com.gemwallet.android.features.stake.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import com.gemwallet.android.features.stake.viewmodels.models.StakeAction
import com.gemwallet.android.features.stake.viewmodels.models.StakeActionUIModel
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Resource

internal fun LazyListScope.stakeActions(
    actions: List<StakeActionUIModel>,
    assetId: AssetId,
    amountAction: AmountTransactionAction,
    onRewards: () -> Unit
) {
    itemsPositioned(actions) { position, item ->
        val onClick = when (item.action) {
            StakeAction.Stake -> {
                { amountAction(AmountParams.Stake.Delegate(assetId)) }
            }
            StakeAction.Freeze -> {
                { amountAction(AmountParams.Stake.Freeze(assetId, Resource.Bandwidth)) }
            }
            StakeAction.Unfreeze -> {
                { amountAction(AmountParams.Stake.Unfreeze(assetId, Resource.Bandwidth)) }
            }
            StakeAction.ClaimRewards -> onRewards
        }
        var showInfo by remember { mutableStateOf(false) }
        ListItem(
            model = item.model,
            listPosition = position,
            modifier = Modifier.clickable(enabled = item.isEnabled) {
                if (item.requiresFrozenBalance) showInfo = true else onClick()
            },
            accessory = { DataBadgeChevron() },
        )
        item.model.info?.let { if (showInfo) InfoBottomSheet(it) { showInfo = false } }
    }
}
