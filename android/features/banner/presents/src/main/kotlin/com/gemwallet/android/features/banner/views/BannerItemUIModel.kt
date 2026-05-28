package com.gemwallet.android.features.banner.views

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.Emoji
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState

internal data class BannerItemUIModel(
    val title: String,
    val subtitle: String,
    val icon: BannerIcon,
    val canClose: Boolean,
)

internal sealed interface BannerIcon {
    @JvmInline value class Emoji(val value: String) : BannerIcon
    @JvmInline value class Url(val value: String) : BannerIcon
    @JvmInline value class Vector(val image: ImageVector) : BannerIcon
}

@Composable
internal fun bannerItemUIModel(
    banner: Banner,
    asset: Asset?,
    getActivationFee: (Asset?) -> String,
): BannerItemUIModel {
    val assetName = asset?.name.orEmpty()
    val (title, subtitle) = when (banner.event) {
        BannerEvent.Stake -> Pair(
            stringResource(R.string.banner_stake_title, assetName),
            stringResource(R.string.banner_stake_description, assetName),
        )
        BannerEvent.AccountActivation -> Pair(
            stringResource(R.string.banner_account_activation_title, assetName),
            stringResource(
                R.string.banner_account_activation_description,
                assetName,
                getActivationFee(asset),
            ),
        )
        BannerEvent.EnableNotifications -> Pair(
            stringResource(R.string.banner_enable_notifications_title, assetName),
            stringResource(R.string.banner_enable_notifications_description),
        )
        BannerEvent.AccountBlockedMultiSignature -> Pair(
            stringResource(R.string.common_warning),
            stringResource(R.string.warnings_multi_signature_blocked, asset?.chain ?: ""),
        )
        BannerEvent.ActivateAsset -> Pair(
            stringResource(R.string.transfer_activate_asset_title),
            stringResource(
                R.string.banner_activate_asset_description,
                assetName,
                asset?.id?.chain?.asset()?.name ?: "",
            ),
        )
        BannerEvent.SuspiciousAsset,
        BannerEvent.Onboarding,
        BannerEvent.TradePerpetuals -> TODO()
    }
    val icon: BannerIcon = when (banner.event) {
        BannerEvent.Stake -> BannerIcon.Emoji(Emoji.moneyBag)
        BannerEvent.AccountBlockedMultiSignature -> BannerIcon.Vector(AppIcons.Warning)
        else -> BannerIcon.Url(
            asset?.getIconUrl()
                ?: "android.resource://com.gemwallet.android/${R.drawable.brandmark}",
        )
    }
    return BannerItemUIModel(
        title = title,
        subtitle = subtitle,
        icon = icon,
        canClose = banner.state != BannerState.AlwaysActive,
    )
}
