package com.gemwallet.android.features.confirm.presents.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemSupportText
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.actionIconGlyphSize
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Asset

@Composable
internal fun GetAssetBottomSheet(
    asset: Asset,
    buyAmount: Int?,
    onDismiss: () -> Unit,
    onAction: (AcquireAssetAction) -> Unit,
) {
    ModalBottomSheet(
        isVisible = true,
        onDismissRequest = onDismiss,
        skipPartiallyExpanded = true,
        title = stringResource(R.string.asset_get_asset, asset.symbol),
    ) {
        Column(modifier = Modifier.padding(bottom = paddingDefault)) {
            GetAssetItem(
                title = stringResource(R.string.wallet_buy),
                subtitle = stringResource(R.string.wallet_pay_with_card_or_bank),
                icon = AppIcons.Buy,
                position = ListPosition.First,
                onClick = { onAction(AcquireAssetAction.Buy(buyAmount)) },
            )
            GetAssetItem(
                title = stringResource(R.string.wallet_swap),
                subtitle = stringResource(R.string.wallet_from_your_wallet_assets),
                icon = AppIcons.SwapVert,
                position = ListPosition.Middle,
                onClick = { onAction(AcquireAssetAction.Swap) },
            )
            GetAssetItem(
                title = stringResource(R.string.wallet_receive),
                subtitle = stringResource(R.string.wallet_transfer_from_another_wallet),
                icon = AppIcons.Receive,
                position = ListPosition.Last,
                onClick = { onAction(AcquireAssetAction.Receive) },
            )
        }
    }
}

@Composable
private fun GetAssetItem(
    title: String,
    subtitle: String,
    icon: ImageVector,
    position: ListPosition,
    onClick: () -> Unit,
) {
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = position,
        leading = { GetAssetIcon(icon = icon) },
        title = { ListItemTitleText(text = title) },
        subtitle = { ListItemSupportText(text = subtitle) },
        trailing = { DataBadgeChevron() },
    )
}

@Composable
private fun GetAssetIcon(icon: ImageVector) {
    Box(
        modifier = Modifier
            .size(listItemIconSize)
            .background(color = MaterialTheme.colorScheme.primary, shape = RoundedCornerShape(12.dp)),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            modifier = Modifier.size(actionIconGlyphSize),
            imageVector = icon,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onPrimary,
        )
    }
}
