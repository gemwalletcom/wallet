package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.Constants
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.DelegationBalanceInfoUIModel
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.pendingColor
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.DelegationState.Activating
import com.wallet.core.primitives.DelegationState.Active
import com.wallet.core.primitives.DelegationState.AwaitingWithdrawal
import com.wallet.core.primitives.DelegationState.Deactivating
import com.wallet.core.primitives.DelegationState.Inactive
import com.wallet.core.primitives.DelegationState.Pending

@Composable
fun DelegationItem(
    assetInfo: AssetInfo,
    delegation: Delegation,
    completedAt: String,
    listPosition: ListPosition,
    onClick: () -> Unit
) {
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        leading = {
            IconWithBadge(
                icon = "${Constants.ASSETS_URL}/blockchains/${delegation.validator.chain.string}/validators/${delegation.validator.id}/logo.png",
                placeholder = delegation.validator.name.firstOrNull()?.toString() ?: delegation.validator.id.firstOrNull()?.toString() ?: "",
            )
        },
        title = {
            ListItemTitleText(text = delegation.validator.name)
        },
        subtitle = {
            val stateColor = delegation.base.state.color()
            val stateText = delegation.stateText()
            Column {
                ListItemSupportText(stateText, color = stateColor)
                when (delegation.base.state) {
                    Pending,
                    Activating,
                    Deactivating -> completedAt.takeIf { it.isNotEmpty() && it != "0" }?.let {
                        ListItemSupportText(it)
                    }
                    Active,
                    Inactive,
                    AwaitingWithdrawal -> Unit
                }
            }
        },
        trailing = {
            val balance = DelegationBalanceInfoUIModel(
                assetInfo = assetInfo,
                delegation = delegation.base,
            )
            Row(verticalAlignment = Alignment.CenterVertically) {
                getBalanceInfo(balance, balance).invoke()
                DataBadgeChevron()
            }
        }
    )
}

@Composable
private fun DelegationState.color() = when (this) {
    Active -> MaterialTheme.colorScheme.tertiary
    Pending,
    Activating,
    Deactivating -> pendingColor
    AwaitingWithdrawal,
    Inactive -> MaterialTheme.colorScheme.error
}

@Composable
private fun Delegation.stateText(): String = stringResource(
    when (base.state) {
        Active -> if (validator.isActive) R.string.stake_active else R.string.stake_inactive
        Pending -> R.string.stake_pending
        Inactive -> R.string.stake_inactive
        Activating -> R.string.stake_activating
        Deactivating -> R.string.stake_deactivating
        AwaitingWithdrawal -> R.string.stake_awaiting_withdrawal
    }
)
