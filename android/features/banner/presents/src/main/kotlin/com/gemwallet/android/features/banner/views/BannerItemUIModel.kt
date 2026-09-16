package com.gemwallet.android.features.banner.views

import com.gemwallet.android.features.banner.views.localization.bannerDescription
import com.gemwallet.android.features.banner.views.localization.bannerTitle
import com.gemwallet.android.ui.components.image.iconResource
import androidx.annotation.DrawableRes
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.Emoji
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerState
import uniffi.gemstone.GemBannerContent
import uniffi.gemstone.GemBannerIcon

internal data class BannerItemUIModel(
    val title: String?,
    val subtitle: String?,
    val icon: BannerIcon?,
    val canClose: Boolean,
)

internal sealed interface BannerIcon {
    @JvmInline value class Emoji(val value: String) : BannerIcon
    @JvmInline value class Url(val value: String) : BannerIcon
    @JvmInline value class Vector(val image: ImageVector) : BannerIcon
    @JvmInline value class Drawable(@param:DrawableRes val id: Int) : BannerIcon
}

@Composable
internal fun bannerItemUIModel(banner: Banner, content: GemBannerContent): BannerItemUIModel = BannerItemUIModel(
    title = content.title?.let { bannerTitle(it) },
    subtitle = content.description?.let { bannerDescription(it) },
    icon = content.icon?.let { bannerIcon(it) },
    canClose = banner.state != BannerState.AlwaysActive,
)

@Composable
private fun bannerIcon(icon: GemBannerIcon): BannerIcon? = when (icon) {
    GemBannerIcon.MoneyBag -> BannerIcon.Emoji(Emoji.moneyBag)
    is GemBannerIcon.Network -> icon.chain.requireChain().iconResource()?.let(BannerIcon::Drawable)
    GemBannerIcon.Warning -> BannerIcon.Vector(AppIcons.Warning)
    GemBannerIcon.Suspicious -> BannerIcon.Drawable(R.drawable.suspicious)
    GemBannerIcon.Bitcoin -> BannerIcon.Vector(AppIcons.CurrencyBitcoin)
    GemBannerIcon.Perpetuals -> BannerIcon.Drawable(R.drawable.ic_perpetuals)
}
