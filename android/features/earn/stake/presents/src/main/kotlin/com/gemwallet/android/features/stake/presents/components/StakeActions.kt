package com.gemwallet.android.features.stake.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.stake.viewmodels.models.StakeActionUIModel
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemStakeAmountInput
import uniffi.gemstone.GemStakeDestination

internal fun LazyListScope.stakeActions(actions: List<StakeActionUIModel>, assetId: AssetId, amountAction: AmountTransactionAction, onRewards: () -> Unit) {
    itemsPositioned(actions) { position, item ->
        val onClick = when (val destination = item.destination) {
            is GemStakeDestination.Amount -> when (val input = destination.input) {
                is GemStakeAmountInput.Stake -> {
                    { amountAction(AmountParams.Stake.Delegate(assetId)) }
                }

                is GemStakeAmountInput.Freeze -> {
                    { amountAction(AmountParams.Stake.Freeze(assetId, input.resource.toPrimitives())) }
                }

                is GemStakeAmountInput.Unfreeze -> {
                    { amountAction(AmountParams.Stake.Unfreeze(assetId, input.resource.toPrimitives())) }
                }

                else -> onRewards
            }

            GemStakeDestination.ClaimRewards -> onRewards
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
