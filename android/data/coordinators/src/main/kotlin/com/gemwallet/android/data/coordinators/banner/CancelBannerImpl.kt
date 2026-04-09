package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.application.banner.coordinators.CancelBanner
import com.gemwallet.android.cases.banners.CancelBannerCase
import com.wallet.core.primitives.Banner

class CancelBannerImpl(
    private val cancelBannerCase: CancelBannerCase,
) : CancelBanner {

    override suspend fun invoke(banner: Banner) {
        cancelBannerCase.cancelBanner(banner)
    }
}
