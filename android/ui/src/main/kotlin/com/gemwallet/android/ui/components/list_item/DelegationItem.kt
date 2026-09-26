package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.localization.stateText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.color
import uniffi.gemstone.GemDelegationListRow

@Composable
fun DelegationItem(row: GemDelegationListRow, listPosition: ListPosition, onClick: () -> Unit) {
    val validator = row.validator
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        leading = {
            IconWithBadge(
                icon = validator.icon,
                placeholder = validator.placeholder,
            )
        },
        title = {
            ListItemTitleText(text = validator.name)
        },
        subtitle = {
            ListItemSupportText(
                row.status.stateText(),
                color = row.status.tone.color(),
            )
        },
        trailing = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                getBalanceInfo(row.balance, row.fiat).invoke()
                DataBadgeChevron()
            }
        },
    )
}
