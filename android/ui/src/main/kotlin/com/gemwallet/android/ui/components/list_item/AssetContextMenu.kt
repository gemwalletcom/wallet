package com.gemwallet.android.ui.components.list_item

import com.gemwallet.android.ui.localization.stringRes
import androidx.annotation.StringRes
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
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.icons.AppIcons
import com.wallet.core.primitives.AssetId
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import uniffi.gemstone.GemAssetMenuAction
import uniffi.gemstone.GemAssetMenuInput
import uniffi.gemstone.assetMenuActions

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
    @get:StringRes val titleRes: Int,
    val icon: @Composable () -> Unit,
    val onClick: () -> Unit,
)

@Composable
fun rememberAssetContextMenuItems(
    assetId: AssetId,
    address: String?,
    isPinned: Boolean,
    isBalanceEnabled: Boolean,
    actions: AssetContextActions,
): List<AssetContextMenuItem> {
    val context = LocalContext.current
    val clipboard = LocalContext.current.clipboardManager()
    return remember(assetId, address, isPinned, isBalanceEnabled, actions) {
        if (actions.isEmpty) return@remember emptyList()
        assetMenuActions(
            GemAssetMenuInput(
                isPinned = isPinned,
                isBalanceEnabled = isBalanceEnabled,
                address = address.orEmpty(),
                offersHide = actions.onHide != null,
                offersAddToWallet = actions.onAddToWallet != null,
            )
        ).mapNotNull { action ->
            when (action) {
                is GemAssetMenuAction.Pin -> actions.onTogglePin?.let { cb ->
                    AssetContextMenuItem(
                        titleRes = action.stringRes(),
                        icon = {
                            if (action.isPinned) Icon(painterResource(R.drawable.keep_off), null)
                            else Icon(AppIcons.PushPin, null)
                        },
                        onClick = { cb(assetId) },
                    )
                }
                GemAssetMenuAction.Hide -> actions.onHide?.let { cb ->
                    AssetContextMenuItem(
                        titleRes = action.stringRes(),
                        icon = { Icon(AppIcons.VisibilityOff, null) },
                        onClick = { cb(assetId) },
                    )
                }
                GemAssetMenuAction.AddToWallet -> actions.onAddToWallet?.let { cb ->
                    AssetContextMenuItem(
                        titleRes = action.stringRes(),
                        icon = { Icon(AppIcons.AddCircleOutlined, null) },
                        onClick = { cb(assetId) },
                    )
                }
                is GemAssetMenuAction.CopyAddress -> AssetContextMenuItem(
                    titleRes = action.stringRes(),
                    icon = { Icon(AppIcons.ContentCopy, null) },
                    onClick = { clipboard.setPlainText(context, action.address) },
                )
            }
        }
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
