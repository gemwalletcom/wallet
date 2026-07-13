package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.DelegationBalanceInfoUIModel
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.Delegation

@Composable
fun DelegationItem(
    assetInfo: AssetInfo,
    delegation: Delegation,
    listPosition: ListPosition,
    onClick: () -> Unit
) {
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        leading = {
            IconWithBadge(
                icon = delegation.validator.getIconUrl(),
                placeholder = delegation.validator.name.firstOrNull()?.toString() ?: delegation.validator.id.firstOrNull()?.toString() ?: "",
            )
        },
        title = {
            ListItemTitleText(text = delegation.validator.name)
        },
        subtitle = {
            ListItemSupportText(
                delegation.base.state.stateText(delegation.validator.isActive),
                color = delegation.base.state.stateColor(),
            )
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
