package com.gemwallet.android.features.assets.presents.asset.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import com.gemwallet.android.features.assets.presents.banner.Banner
import com.gemwallet.android.ui.open
import uniffi.gemstone.GemAssetBanner
import uniffi.gemstone.GemAssetBannerDestination
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemTransferData

@Composable
internal fun BannerItem(banner: GemAssetBanner, onStake: () -> Unit, onActivate: (GemTransferData) -> Unit, onOpenPerpetuals: () -> Unit, onClose: (GemBannerKey) -> Unit) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    Banner(
        banner = banner.row,
        destination = banner.destination,
        onSelect = { destination ->
            when (destination) {
                GemAssetBannerDestination.Stake -> onStake()
                is GemAssetBannerDestination.ActivateAsset -> onActivate(destination.transfer)
                GemAssetBannerDestination.Perpetuals -> onOpenPerpetuals()
                is GemAssetBannerDestination.Url -> uriHandler.open(context, destination.url)
            }
        },
        onClose = onClose,
    )
}
