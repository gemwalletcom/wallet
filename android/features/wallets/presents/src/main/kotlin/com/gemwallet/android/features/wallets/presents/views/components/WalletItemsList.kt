package com.gemwallet.android.features.wallets.presents.views.components

import androidx.annotation.StringRes
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.DropDownContextItem
import com.gemwallet.android.ui.components.list_item.WalletItem
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.WalletId

internal fun LazyListScope.wallets(
    wallets: List<WalletDataAggregate>,
    longPressedWallet: MutableState<String>,
    isPinned: Boolean = false,
    onEdit: (WalletId) -> Unit,
    onSelectWallet: (WalletId) -> Unit,
    onDeleteWallet: (WalletId) -> Unit,
    onTogglePin: (WalletId) -> Unit,
) {
    if (isPinned && wallets.isNotEmpty()) {
        pinnedHeader()
    }
    itemsIndexed(items = wallets, key = { _, item -> item.id }) { index, item ->
        val walletId = WalletId(item.id)

        DropDownContextItem(
            isExpanded = longPressedWallet.value == item.id,
            onDismiss = { longPressedWallet.value = "" },
            content = {
                WalletItem(
                    id = item.id,
                    name = item.name,
                    walletChain = item.walletChain,
                    walletAddress = item.walletAddress,
                    isCurrent = item.isCurrent,
                    type = item.type,
                    listPosition = ListPosition.getPosition(index, wallets.size),
                    onEdit = { onEdit(walletId) },
                    modifier = it
                )
            },
            menuItems = {
                WalletDropDownItem(
                    if (item.isPinned) R.string.common_unpin else R.string.common_pin,
                    if (item.isPinned) R.drawable.keep_off else AppIcons.PushPin,
                ) {
                    onTogglePin(walletId)
                    longPressedWallet.value = ""
                }
                WalletDropDownItem(R.string.common_wallet, AppIcons.Settings) {
                    onEdit(walletId)
                    longPressedWallet.value = ""
                }
                WalletDropDownItem(R.string.common_delete, AppIcons.Delete, MaterialTheme.colorScheme.error) {
                    onDeleteWallet(walletId)
                    longPressedWallet.value = ""
                }
            },
            onLongClick = { longPressedWallet.value = item.id }
        ) { onSelectWallet(walletId) }
    }
}

@Composable
private fun WalletDropDownItem(
    @StringRes text: Int,
    icon: Any,
    color: Color = Color.Unspecified,
    onClick: () -> Unit,
) {
    val text = stringResource(text)
    DropdownMenuItem(
        text = {
            Text(text = text, color = color)
        },
        trailingIcon = {
            when (icon) {
                is ImageVector -> Icon(
                    imageVector = icon,
                    tint = color,
                    contentDescription = text
                )
                is Int -> Icon(painterResource(icon), text)
            }

        },
        onClick = onClick,
    )
}
