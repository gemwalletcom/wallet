package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.application.banner.cases.ApplyBannerAction
import com.gemwallet.android.domains.banner.BannerAction
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Banner
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemBannerAction
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemBannerService

class ApplyBannerActionImpl(
    private val bannerService: GemBannerService,
) : ApplyBannerAction {

    override suspend fun invoke(banner: Banner, action: BannerAction) = withContext(Dispatchers.IO) {
        bannerService.applyAction(banner.toGemKey(), action.toGem())
    }
}

private fun Banner.toGemKey() = GemBannerKey(
    walletId = walletId?.id,
    assetId = asset?.id?.toIdentifier(),
    event = event.toJson(),
)

private fun BannerAction.toGem(): GemBannerAction = when (this) {
    is BannerAction.Event -> GemBannerAction.Event(event.toJson())
    BannerAction.Close -> GemBannerAction.Close
}
