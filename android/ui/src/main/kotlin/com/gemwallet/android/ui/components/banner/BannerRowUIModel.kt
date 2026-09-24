package com.gemwallet.android.ui.components.banner

import android.content.Context
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.localization.bannerDescription
import com.gemwallet.android.ui.localization.bannerTitle
import com.gemwallet.android.ui.style.image
import com.wallet.core.primitives.BannerState
import uniffi.gemstone.GemBannerButton
import uniffi.gemstone.GemBannerDestination
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemBannerRow
import uniffi.gemstone.GemBannerStyle

data class BannerItemUIModel(val title: String?, val subtitle: String?, val icon: ListItemImage?, val canClose: Boolean, val destination: GemBannerDestination?, val style: GemBannerStyle, val buttons: List<GemBannerButton>)

data class BannerRowUIModel(val key: GemBannerKey, val model: BannerItemUIModel)

fun GemBannerRow.uiModel(context: Context): BannerRowUIModel = BannerRowUIModel(
    key = key,
    model = BannerItemUIModel(
        title = content.title?.let { bannerTitle(context, it) },
        subtitle = content.description?.let { bannerDescription(context, it) },
        icon = content.icon?.image(),
        canClose = content.canClose,
        destination = content.destination,
        style = content.style,
        buttons = content.buttons,
    ),
)
