package com.gemwallet.android.features.wallet.presents.components

import androidx.annotation.StringRes
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.DropDownContextItem
import com.gemwallet.android.ui.components.list_item.WalletItem
import com.gemwallet.android.ui.components.list_item.pinnedHeader
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSection
import uniffi.gemstone.GemWalletSectionKind

internal fun LazyListScope.wallets(
    section: GemWalletSection,
    longPressedWallet: MutableState<String>,
    onEdit: (GemWalletRow) -> Unit,
    onSelectWallet: (GemWalletRow) -> Unit,
    onDeleteWallet: (GemWalletRow) -> Unit,
    onTogglePin: (GemWalletRow) -> Unit,
) {
    if (section.kind == GemWalletSectionKind.PINNED) {
        pinnedHeader()
    }
    itemsIndexed(items = section.rows, key = { _, row -> row.id }) { index, row ->
        DropDownContextItem(
            isExpanded = longPressedWallet.value == row.id,
            onDismiss = { longPressedWallet.value = "" },
            content = {
                WalletItem(
                    row = row,
                    isCurrent = row.isCurrent,
                    listPosition = ListPosition.getPosition(index, section.rows.size),
                    onEdit = { onEdit(row) },
                    modifier = it,
                )
            },
            menuItems = {
                WalletDropDownItem(
                    if (row.isPinned) R.string.common_unpin else R.string.common_pin,
                    if (row.isPinned) AppIcons.KeepOff else AppIcons.PushPin,
                ) {
                    onTogglePin(row)
                    longPressedWallet.value = ""
                }
                WalletDropDownItem(R.string.common_wallet, AppIcons.Settings) {
                    onEdit(row)
                    longPressedWallet.value = ""
                }
                HorizontalDivider()
                WalletDropDownItem(R.string.common_delete, AppIcons.Delete, MaterialTheme.colorScheme.error) {
                    onDeleteWallet(row)
                    longPressedWallet.value = ""
                }
            },
            onLongClick = { longPressedWallet.value = row.id },
        ) { onSelectWallet(row) }
    }
}

@Composable
private fun WalletDropDownItem(@StringRes text: Int, icon: ImageVector, color: Color = Color.Unspecified, onClick: () -> Unit) {
    val text = stringResource(text)
    DropdownMenuItem(
        text = {
            Text(text = text, color = color)
        },
        trailingIcon = {
            Icon(
                imageVector = icon,
                tint = color,
                contentDescription = text,
            )
        },
        onClick = onClick,
    )
}
