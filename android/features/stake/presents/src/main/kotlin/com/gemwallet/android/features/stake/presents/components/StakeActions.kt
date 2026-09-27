package com.gemwallet.android.features.stake.presents.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.model.toAmountParams
import com.gemwallet.android.ui.components.InfoBottomSheet
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.listItemModel
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemDelegationAmountInput
import uniffi.gemstone.GemStakeAction
import uniffi.gemstone.GemStakeActionItem
import uniffi.gemstone.GemStakeDestination
import uniffi.gemstone.GemTransferData

internal fun LazyListScope.stakeActions(actions: List<GemStakeActionItem>, assetId: AssetId, amountAction: AmountTransactionAction, onConfirm: (GemTransferData) -> Unit) {
    itemsPositioned(actions) { position, item ->
        val model = item.row.listItemModel(LocalContext.current) ?: ListItemModel(title = "")
        var showInfo by remember { mutableStateOf(false) }
        val onClick: (() -> Unit)? = when (val action = item.action) {
            is GemStakeAction.Open -> when (val destination = action.destination) {
                is GemStakeDestination.Amount -> {
                    { amountAction(GemDelegationAmountInput.Stake(destination.input).toAmountParams(assetId)) }
                }

                is GemStakeDestination.Confirm -> {
                    { onConfirm(destination.transfer) }
                }
            }

            GemStakeAction.FrozenBalanceInfo -> {
                { showInfo = true }
            }

            GemStakeAction.Disabled -> null
        }
        ListItem(
            model = model,
            listPosition = position,
            modifier = Modifier.clickable(enabled = onClick != null) { onClick?.invoke() },
            accessory = { DataBadgeChevron() },
        )
        model.info?.let { if (showInfo) InfoBottomSheet(it) { showInfo = false } }
    }
}
