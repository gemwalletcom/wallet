package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.localization.stateText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.style.color
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Delegation
import uniffi.gemstone.delegationListRow

@Composable
fun DelegationItem(assetInfo: AssetInfo, delegation: Delegation, validator: ValidatorRowUIModel, listPosition: ListPosition, onClick: () -> Unit) {
    val row = remember(delegation, assetInfo) {
        delegationListRow(
            delegation.toGem(),
            assetInfo.asset.toGem(),
            assetInfo.price?.price?.price,
            (assetInfo.price?.currency ?: Currency.USD).toGem(),
        )
    }
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        leading = {
            IconWithBadge(
                icon = validator.imageUrl,
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
                getBalanceInfo(row.balance.text(), row.fiat?.text().orEmpty(), !row.hasBalance).invoke()
                DataBadgeChevron()
            }
        },
    )
}
