package com.gemwallet.android.ui.components.banner

import android.content.Context
import com.gemwallet.android.AppUrl
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.localization.bannerDescription
import com.gemwallet.android.ui.localization.bannerTitle
import com.gemwallet.android.ui.style.image
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerState
import uniffi.gemstone.GemBannerDestination
import uniffi.gemstone.GemBannerLink
import uniffi.gemstone.GemBannerRow
import uniffi.gemstone.GemTransferData

data class BannerItemUIModel(val title: String?, val subtitle: String?, val icon: ListItemImage?, val canClose: Boolean, val destination: BannerDestination?)

sealed interface BannerDestination {
    data object Stake : BannerDestination
    data class Activate(val transfer: GemTransferData) : BannerDestination
    data object Perpetuals : BannerDestination
    data class OpenUrl(val url: String) : BannerDestination
}

data class BannerRowUIModel(val banner: Banner, val model: BannerItemUIModel)

fun GemBannerRow.uiModel(context: Context): BannerRowUIModel = BannerRowUIModel(
    banner = banner.toPrimitives(),
    model = BannerItemUIModel(
        title = content.title?.let { bannerTitle(context, it) },
        subtitle = content.description?.let { bannerDescription(context, it) },
        icon = content.icon?.image(),
        canClose = banner.state.toPrimitives() != BannerState.AlwaysActive,
        destination = content.destination?.destination(),
    ),
)

private fun GemBannerDestination.destination(): BannerDestination = when (this) {
    GemBannerDestination.Stake -> BannerDestination.Stake
    is GemBannerDestination.ActivateAsset -> BannerDestination.Activate(transfer)
    GemBannerDestination.Perpetuals -> BannerDestination.Perpetuals
    is GemBannerDestination.Url -> BannerDestination.OpenUrl(link.url())
}

private fun GemBannerLink.url(): String = when (this) {
    is GemBannerLink.Docs -> AppUrl.docs(item)
    is GemBannerLink.External -> url
}
