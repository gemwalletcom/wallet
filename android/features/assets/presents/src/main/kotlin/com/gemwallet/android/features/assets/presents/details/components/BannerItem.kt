package com.gemwallet.android.features.assets.presents.details.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import com.gemwallet.android.features.banner.views.BannerScene
import com.gemwallet.android.ui.components.banner.BannerRowUIModel
import com.gemwallet.android.ui.open
import uniffi.gemstone.GemBannerDestination
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemTransferData

@Composable
internal fun BannerItem(banner: BannerRowUIModel, onStake: () -> Unit, onActivate: (GemTransferData) -> Unit, onOpenPerpetuals: () -> Unit, onClose: (GemBannerKey) -> Unit) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    BannerScene(
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
