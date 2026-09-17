package com.gemwallet.android.ui.components.banner

import android.content.Context
import com.gemwallet.android.AppUrl
import com.gemwallet.android.domains.banner.BannerRow
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.localization.bannerDescription
import com.gemwallet.android.ui.localization.bannerTitle
import com.gemwallet.android.ui.theme.Emoji
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerState
import uniffi.gemstone.GemBannerIcon
import uniffi.gemstone.GemBannerLink

data class BannerItemUIModel(
    val title: String?,
    val subtitle: String?,
    val icon: ListItemImage?,
    val canClose: Boolean,
    val url: String?,
)

data class BannerRowUIModel(
    val banner: Banner,
    val model: BannerItemUIModel,
)

fun BannerRow.uiModel(context: Context): BannerRowUIModel = BannerRowUIModel(
    banner = banner,
    model = BannerItemUIModel(
        title = content.title?.let { bannerTitle(context, it) },
        subtitle = content.description?.let { bannerDescription(context, it) },
        icon = content.icon?.image(),
        canClose = banner.state != BannerState.AlwaysActive,
        url = content.link?.url(),
    ),
)

private fun GemBannerIcon.image(): ListItemImage = when (this) {
    GemBannerIcon.MoneyBag -> ListItemImage.Emoji(Emoji.moneyBag)
    is GemBannerIcon.Network -> ListItemImage.Asset(AssetId(chain.requireChain()))
    GemBannerIcon.Warning -> ListItemImage.Symbol(ListItemSymbol.Warning)
    GemBannerIcon.Suspicious -> ListItemImage.Drawable(R.drawable.suspicious)
    GemBannerIcon.Bitcoin -> ListItemImage.Symbol(ListItemSymbol.CurrencyBitcoin)
    GemBannerIcon.Perpetuals -> ListItemImage.Drawable(R.drawable.ic_perpetuals)
}

private fun GemBannerLink.url(): String = when (this) {
    is GemBannerLink.Docs -> AppUrl.docs(item)
    is GemBannerLink.External -> url
}
