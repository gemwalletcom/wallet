package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.application.banner.cases.GetBannerContent
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Banner
import uniffi.gemstone.GemBannerContent
import uniffi.gemstone.GemBannerService

class GetBannerContentImpl(
    private val bannerService: GemBannerService,
) : GetBannerContent {

    override fun invoke(banner: Banner): GemBannerContent =
        bannerService.bannerContent(banner.event.toJson(), banner.asset?.toJson())
}
