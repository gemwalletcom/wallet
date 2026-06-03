package com.gemwallet.android.features.transfer_amount.presents.dialogs

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.perpetual.color
import com.gemwallet.android.ui.components.perpetual.text
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.paddingMiddle
import com.gemwallet.android.ui.theme.space0
import com.gemwallet.android.ui.theme.space6
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualDirection

@Composable
internal fun OpenPositionItem(
    asset: Asset,
    direction: PerpetualDirection,
    leverage: Int,
    sizeText: String,
    modifier: Modifier = Modifier,
    listPosition: ListPosition = ListPosition.Single,
) {
    ListItem(
        modifier = modifier,
        listPosition = listPosition,
        minHeight = ListItemDefaults.iconMinHeight,
        contentPadding = adaptivePadding(default = paddingMiddle, compact = space6),
        titleSubtitleSpacing = space0,
        leading = { AssetIcon(asset) },
        title = { ListItemTitleText(asset.symbol) },
        subtitle = { ListItemSupportText(direction.text(leverage), color = direction.color()) },
        trailing = {
            ListItemTitleText(text = sizeText, color = MaterialTheme.colorScheme.onSurface)
        },
    )
}
