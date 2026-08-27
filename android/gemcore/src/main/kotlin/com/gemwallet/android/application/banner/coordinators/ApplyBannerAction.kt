package com.gemwallet.android.application.banner.coordinators

import com.gemwallet.android.domains.banner.BannerAction
import com.wallet.core.primitives.Banner

interface ApplyBannerAction {
    suspend operator fun invoke(banner: Banner, action: BannerAction)
}
