package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.application.banner.cases.ApplyBannerAction
import com.gemwallet.android.cases.banners.BannerActionCase
import com.gemwallet.android.domains.banner.BannerAction
import com.wallet.core.primitives.Banner

class ApplyBannerActionImpl(
    private val bannerActionCase: BannerActionCase,
) : ApplyBannerAction {

    override suspend fun invoke(banner: Banner, action: BannerAction) {
        bannerActionCase.applyBannerAction(banner, action)
    }
}
