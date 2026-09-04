package com.gemwallet.android.features.banner.views

import androidx.annotation.DrawableRes
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.Emoji
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerState
import uniffi.gemstone.GemBannerAmount
import uniffi.gemstone.GemBannerContent
import uniffi.gemstone.GemBannerDescription
import uniffi.gemstone.GemBannerIcon
import uniffi.gemstone.GemBannerTitle

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
private fun bannerTitle(title: GemBannerTitle): String = when (title) {
    is GemBannerTitle.Stake -> stringResource(R.string.banner_stake_title, title.assetName)
    GemBannerTitle.AccountActivation -> stringResource(R.string.banner_account_activation_title)
    GemBannerTitle.Warning -> stringResource(R.string.common_warning)
    GemBannerTitle.ActivateAsset -> stringResource(R.string.transfer_activate_asset_title)
    GemBannerTitle.SuspiciousAsset -> stringResource(R.string.banner_asset_status_title)
    GemBannerTitle.Onboarding -> stringResource(R.string.banner_onboarding_title)
    GemBannerTitle.TradePerpetuals -> stringResource(R.string.banner_perpetuals_title)
}

@Composable
private fun bannerDescription(description: GemBannerDescription): String = when (description) {
    is GemBannerDescription.Stake -> stringResource(R.string.banner_stake_description, description.assetSymbol)
    is GemBannerDescription.AccountActivation -> stringResource(
        R.string.banner_account_activation_description,
        description.networkName,
        formatAmount(description.fee),
    )
    is GemBannerDescription.MultiSignatureBlocked -> stringResource(R.string.warnings_multi_signature_blocked, description.networkName)
    is GemBannerDescription.ActivateAsset -> stringResource(
        R.string.banner_activate_asset_description,
        description.assetSymbol,
        description.networkName,
    )
    GemBannerDescription.SuspiciousAsset -> stringResource(R.string.banner_asset_status_description)
    GemBannerDescription.Onboarding -> stringResource(R.string.banner_onboarding_description)
    GemBannerDescription.TradePerpetuals -> stringResource(R.string.banner_perpetuals_description)
}

@Composable
private fun bannerIcon(icon: GemBannerIcon): BannerIcon? = when (icon) {
    GemBannerIcon.MoneyBag -> BannerIcon.Emoji(Emoji.moneyBag)
    is GemBannerIcon.Network -> BannerIcon.Url(icon.chain.requireChain().getIconUrl())
    GemBannerIcon.Warning -> BannerIcon.Vector(AppIcons.Warning)
    GemBannerIcon.Suspicious -> BannerIcon.Drawable(R.drawable.suspicious)
    GemBannerIcon.Bitcoin -> BannerIcon.Vector(AppIcons.CurrencyBitcoin)
    GemBannerIcon.Perpetuals -> BannerIcon.Drawable(R.drawable.ic_perpetuals)
}

private fun formatAmount(amount: GemBannerAmount): String = ValueFormatter(style = ValueFormatter.Style.Auto)
    .string(amount.value, decimals = amount.decimals, currency = amount.symbol)
