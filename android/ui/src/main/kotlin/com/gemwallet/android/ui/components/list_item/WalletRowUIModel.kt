package com.gemwallet.android.ui.components.list_item

import android.content.Context
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.image.walletImageModel
import com.gemwallet.android.ui.localization.string
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemWalletPlaceholder
import uniffi.gemstone.GemWalletRow

data class WalletRowUIModel(
    val id: String,
    val name: String,
    val subtitle: String,
    val icon: Any?,
    val supportIcon: String?,
)

fun GemWalletRow.uiModel(context: Context) = WalletRowUIModel(
    id = id,
    name = name,
    subtitle = subtitle.string(context),
    icon = walletImageModel(context, imageUrl) ?: placeholder.iconModel(),
    supportIcon = supportIcon(),
)

fun GemWalletRow.listItemImage(): ListItemImage = imageUrl?.takeIf { it.isNotEmpty() }?.let { ListItemImage.Stored(it) } ?: when (val placeholder = placeholder) {
    GemWalletPlaceholder.Multicoin -> ListItemImage.Drawable(R.drawable.multicoin_wallet)
    is GemWalletPlaceholder.Chain -> ListItemImage.Asset(AssetId(placeholder.chain.toChain()))
}

fun GemWalletPlaceholder.iconModel(): Any? = when (this) {
    GemWalletPlaceholder.Multicoin -> R.drawable.multicoin_wallet
    is GemWalletPlaceholder.Chain -> chain.toChain().iconModel()
}

fun GemWalletRow.supportIcon(): String? = if (showsWatchBadge) {
    "android.resource://com.gemwallet.android/drawable/${R.drawable.watch_badge}"
} else {
    null
}
