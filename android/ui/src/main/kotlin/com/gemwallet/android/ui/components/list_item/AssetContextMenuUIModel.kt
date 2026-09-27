package com.gemwallet.android.ui.components.list_item

import android.content.Context
import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.runtime.Immutable
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setCopy
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.iconRes
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemAssetMenuAction
import uniffi.gemstone.GemAssetMenuInput
import uniffi.gemstone.addressCopy
import uniffi.gemstone.assetMenuRows

@Immutable
class AssetContextMenuItem(@get:StringRes val titleRes: Int, @get:DrawableRes val iconRes: Int, val onClick: () -> Unit)

fun assetContextMenuItems(context: Context, assetId: AssetId, address: String?, isPinned: Boolean, isBalanceEnabled: Boolean, actions: AssetContextActions): List<AssetContextMenuItem> {
    if (actions.isEmpty) return emptyList()
    val clipboard = context.clipboardManager()
    return assetMenuRows(
        GemAssetMenuInput(
            isPinned = isPinned,
            isBalanceEnabled = isBalanceEnabled,
            address = address.orEmpty(),
            offersHide = actions.onHide != null,
            offersAddToWallet = actions.onAddToWallet != null,
        ),
    ).mapNotNull { row ->
        val onClick: (() -> Unit)? = when (val action = row.action) {
            is GemAssetMenuAction.Pin -> actions.onTogglePin?.let { { it(assetId) } }

            GemAssetMenuAction.Hide -> actions.onHide?.let { { it(assetId) } }

            GemAssetMenuAction.AddToWallet -> actions.onAddToWallet?.let { { it(assetId) } }

            is GemAssetMenuAction.CopyAddress -> {
                { clipboard.setCopy(context, addressCopy(assetId.chain.string, action.address)) }
            }
        }
        onClick?.let { AssetContextMenuItem(titleRes = row.action.stringRes(), iconRes = row.icon.iconRes(), onClick = it) }
    }
}
