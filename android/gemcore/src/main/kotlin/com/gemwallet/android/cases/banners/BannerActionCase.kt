package com.gemwallet.android.cases.banners

import com.gemwallet.android.domains.banner.BannerAction
import com.wallet.core.primitives.Banner

interface BannerActionCase {
    suspend fun applyBannerAction(banner: Banner, action: BannerAction)
}
