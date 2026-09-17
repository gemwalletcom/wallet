package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.wallet.core.primitives.AssetId

@Immutable
data class AssetContextActions(
    val onTogglePin: ((AssetId) -> Unit)? = null,
    val onHide: ((AssetId) -> Unit)? = null,
    val onAddToWallet: ((AssetId) -> Unit)? = null,
) {
    val isEmpty: Boolean
        get() = onTogglePin == null && onHide == null && onAddToWallet == null

    companion object {
        val Empty = AssetContextActions()
    }
}

@Composable
fun rememberAssetContextMenuItems(
    assetId: AssetId,
    address: String?,
    isPinned: Boolean,
    isBalanceEnabled: Boolean,
    actions: AssetContextActions,
): List<AssetContextMenuItem> {
    val context = LocalContext.current
    return remember(assetId, address, isPinned, isBalanceEnabled, actions) {
        assetContextMenuItems(context, assetId, address, isPinned, isBalanceEnabled, actions)
    }
}

@Composable
private fun ColumnScope.AssetContextMenuItems(
    items: List<AssetContextMenuItem>,
    onDismiss: () -> Unit,
) {
    items.forEach { item ->
        DropdownMenuItem(
            text = { Text(stringResource(item.titleRes)) },
            trailingIcon = { Icon(painterResource(item.iconRes), null) },
            onClick = {
                item.onClick()
                onDismiss()
            },
        )
    }
}

@Composable
private fun AssetMenuRow(
    items: List<AssetContextMenuItem>,
    isExpanded: Boolean,
    onClick: () -> Unit,
    onLongClick: () -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable (Modifier) -> Unit,
) {
    if (items.isEmpty()) {
        content(modifier.clickable(onClick = onClick))
        return
    }
    DropDownContextItem(
        modifier = modifier,
        isExpanded = isExpanded,
        onDismiss = onDismiss,
        menuItems = { AssetContextMenuItems(items, onDismiss) },
        content = content,
        onLongClick = onLongClick,
        onClick = onClick,
    )
}

@Composable
fun AssetContextMenuRow(
    assetId: AssetId,
    address: String?,
    isPinned: Boolean,
    isBalanceEnabled: Boolean,
    longPressed: MutableState<AssetId?>,
    actions: AssetContextActions,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable (Modifier) -> Unit,
) {
    val items = rememberAssetContextMenuItems(assetId, address, isPinned, isBalanceEnabled, actions)
    AssetMenuRow(
        modifier = modifier,
        items = items,
        isExpanded = longPressed.value == assetId,
        onClick = onClick,
        onLongClick = { longPressed.value = assetId },
        onDismiss = { longPressed.value = null },
        content = content,
    )
}
