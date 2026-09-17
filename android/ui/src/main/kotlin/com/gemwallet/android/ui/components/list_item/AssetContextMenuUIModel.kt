package com.gemwallet.android.ui.components.list_item

import android.content.Context
import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.runtime.Immutable
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.localization.stringRes
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetMenuAction
import uniffi.gemstone.GemAssetMenuInput
import uniffi.gemstone.assetMenuActions

@Immutable
class AssetContextMenuItem(
    @get:StringRes val titleRes: Int,
    @get:DrawableRes val iconRes: Int,
    val onClick: () -> Unit,
)

fun assetContextMenuItems(
    context: Context,
    assetId: AssetId,
    address: String?,
    isPinned: Boolean,
    isBalanceEnabled: Boolean,
    actions: AssetContextActions,
): List<AssetContextMenuItem> {
    if (actions.isEmpty) return emptyList()
    val clipboard = context.clipboardManager()
    return assetMenuActions(
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
                    iconRes = if (action.isPinned) R.drawable.keep_off else R.drawable.ic_push_pin,
                    onClick = { cb(assetId) },
                )
            }
            GemAssetMenuAction.Hide -> actions.onHide?.let { cb ->
                AssetContextMenuItem(titleRes = action.stringRes(), iconRes = R.drawable.ic_visibility_off, onClick = { cb(assetId) })
            }
            GemAssetMenuAction.AddToWallet -> actions.onAddToWallet?.let { cb ->
                AssetContextMenuItem(titleRes = action.stringRes(), iconRes = R.drawable.ic_add_circle_outlined, onClick = { cb(assetId) })
            }
            is GemAssetMenuAction.CopyAddress -> AssetContextMenuItem(
                titleRes = action.stringRes(),
                iconRes = R.drawable.ic_content_copy,
                onClick = { clipboard.setPlainText(context, action.address) },
            )
        }
    }
}
