package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import com.gemwallet.android.features.banner.views.BannersScene
import com.gemwallet.android.ui.components.banner.BannerDestination
import com.gemwallet.android.ui.components.banner.BannerRowUIModel
import com.gemwallet.android.ui.open
import com.wallet.core.primitives.Banner
import uniffi.gemstone.GemTransferData

@Composable
internal fun BannerItem(
    banners: List<BannerRowUIModel>,
    onStake: () -> Unit,
    onActivate: (GemTransferData) -> Unit,
    onOpenPerpetuals: () -> Unit,
    onClose: (Banner) -> Unit,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    BannersScene(
        banners = banners,
        onSelect = { destination ->
            when (destination) {
                BannerDestination.Stake -> onStake()
                is BannerDestination.Activate -> onActivate(destination.transfer)
                BannerDestination.Perpetuals -> onOpenPerpetuals()
                is BannerDestination.OpenUrl -> uriHandler.open(context, destination.url)
            }
        },
        onClose = onClose,
    )
}
