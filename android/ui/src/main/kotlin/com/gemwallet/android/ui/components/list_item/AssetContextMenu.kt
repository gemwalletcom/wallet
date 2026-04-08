package com.gemwallet.android.ui.components.list_item

import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AddCircleOutline
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.PushPin
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.setPlainText
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

@Immutable
class AssetContextMenuItem(
    @StringRes val titleRes: Int,
    val icon: @Composable () -> Unit,
    val onClick: () -> Unit,
)

@Composable
fun rememberAssetContextMenuItems(
    assetId: AssetId,
    address: String?,
    isPinned: Boolean,
    isEnabled: Boolean,
    actions: AssetContextActions,
): List<AssetContextMenuItem> {
    val context = LocalContext.current
    val clipboard = LocalClipboard.current.nativeClipboard
    return remember(assetId, address, isPinned, isEnabled, actions) {
        if (actions.isEmpty) return@remember emptyList()
        listOfNotNull(
            actions.onTogglePin?.let { cb ->
                AssetContextMenuItem(
                    titleRes = if (isPinned) R.string.common_unpin else R.string.common_pin,
                    icon = {
                        if (isPinned) Icon(painterResource(R.drawable.keep_off), null)
                        else Icon(Icons.Default.PushPin, null)
                    },
                    onClick = { cb(assetId) },
                )
            },
            address?.takeUnless(String::isEmpty)?.let { addr ->
                AssetContextMenuItem(
                    titleRes = R.string.wallet_copy_address,
                    icon = { Icon(Icons.Default.ContentCopy, null) },
                    onClick = { clipboard.setPlainText(context, addr) },
                )
            },
            actions.onHide?.let { cb ->
                AssetContextMenuItem(
                    titleRes = R.string.common_hide,
                    icon = { Icon(Icons.Default.VisibilityOff, null) },
                    onClick = { cb(assetId) },
                )
            },
            actions.onAddToWallet?.takeUnless { isEnabled }?.let { cb ->
                AssetContextMenuItem(
                    titleRes = R.string.asset_add_to_wallet,
                    icon = { Icon(Icons.Default.AddCircleOutline, null) },
                    onClick = { cb(assetId) },
                )
            },
        )
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
            trailingIcon = item.icon,
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
    isEnabled: Boolean,
    longPressed: MutableState<AssetId?>,
    actions: AssetContextActions,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable (Modifier) -> Unit,
) {
    val items = rememberAssetContextMenuItems(assetId, address, isPinned, isEnabled, actions)
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
