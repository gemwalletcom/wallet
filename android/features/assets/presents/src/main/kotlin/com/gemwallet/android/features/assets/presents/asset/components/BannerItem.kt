package com.gemwallet.android.features.assets.presents.asset.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import com.gemwallet.android.features.assets.presents.banner.Banner
import com.gemwallet.android.ui.open
import uniffi.gemstone.GemBannerDestination
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemBannerRow
import uniffi.gemstone.GemTransferData

@Composable
internal fun BannerItem(banner: GemBannerRow, onStake: () -> Unit, onActivate: (GemTransferData) -> Unit, onOpenPerpetuals: () -> Unit, onClose: (GemBannerKey) -> Unit) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    Banner(
        banner = banner,
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
