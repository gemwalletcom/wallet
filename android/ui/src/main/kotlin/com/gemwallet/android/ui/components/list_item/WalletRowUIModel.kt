package com.gemwallet.android.ui.components.list_item

import android.content.Context
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.image.walletImageModel
import com.gemwallet.android.ui.localization.string
import uniffi.gemstone.GemWalletPlaceholder
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSection
import uniffi.gemstone.GemWalletSectionKind

data class WalletRowUIModel(val id: String, val name: String, val subtitle: String, val icon: Any?, val supportIcon: String?)

data class WalletSectionUIModel(val kind: GemWalletSectionKind, val rows: List<WalletRowUIModel>)

fun GemWalletRow.uiModel(context: Context) = WalletRowUIModel(
    id = id,
    name = name,
    subtitle = subtitle.string(context),
    icon = walletImageModel(context, imageUrl) ?: placeholder.iconModel(),
    supportIcon = supportIcon(),
)

fun GemWalletSection.uiModel(context: Context) = WalletSectionUIModel(kind = kind, rows = rows.map { it.uiModel(context) })

fun GemWalletRow.listItemImage(): ListItemImage = walletListItemImage(imageUrl, placeholder)

fun walletListItemImage(imageUrl: String?, placeholder: GemWalletPlaceholder): ListItemImage = imageUrl?.takeIf { it.isNotEmpty() }?.let { ListItemImage.Stored(it) } ?: when (placeholder) {
    GemWalletPlaceholder.Multicoin -> ListItemImage.Drawable(R.drawable.multicoin_wallet, style = ListItemImageStyle.Avatar)
    is GemWalletPlaceholder.Chain -> ListItemImage.Asset(placeholder.chain)
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
