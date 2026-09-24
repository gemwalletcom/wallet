package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import com.gemwallet.android.features.banner.views.BannersScene
import com.gemwallet.android.ui.components.banner.BannerRowUIModel
import com.gemwallet.android.ui.open
import uniffi.gemstone.GemBannerDestination
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemTransferData

@Composable
internal fun BannerItem(banners: List<BannerRowUIModel>, onStake: () -> Unit, onActivate: (GemTransferData) -> Unit, onOpenPerpetuals: () -> Unit, onClose: (GemBannerKey) -> Unit) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    BannersScene(
        banners = banners,
        onSelect = { destination ->
            when (destination) {
                GemBannerDestination.Stake -> onStake()
                is GemBannerDestination.ActivateAsset -> onActivate(destination.transfer)
                GemBannerDestination.Perpetuals -> onOpenPerpetuals()
                is GemBannerDestination.Url -> uriHandler.open(context, destination.url)
            }
        },
        onClose = onClose,
    )
}
