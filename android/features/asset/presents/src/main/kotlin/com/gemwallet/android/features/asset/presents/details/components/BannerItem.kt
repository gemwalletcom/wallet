package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.banner.views.BannersScene
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.components.banner.BannerRowUIModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent

@Composable
internal fun BannerItem(
    assetInfo: AssetInfo,
    banners: List<BannerRowUIModel>,
    onStake: (AssetId) -> Unit,
    onActivate: () -> Unit,
    onOpenPerpetuals: () -> Unit,
    onClose: (Banner) -> Unit,
) {
    BannersScene(
        banners = banners,
        onSelect = {
            when (it.event) {
                BannerEvent.Stake -> onStake(assetInfo.asset.id)
                BannerEvent.ActivateAsset -> onActivate()

                BannerEvent.TradePerpetuals -> onOpenPerpetuals()
                BannerEvent.AccountActivation,
                BannerEvent.AccountBlockedMultiSignature,
                BannerEvent.SuspiciousAsset,
                BannerEvent.Onboarding -> Unit
            }
        },
        onClose = onClose,
    )
}
