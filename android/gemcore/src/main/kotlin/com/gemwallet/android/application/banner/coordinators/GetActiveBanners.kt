package com.gemwallet.android.application.banner.coordinators

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner

interface GetActiveBanners {
    suspend fun getActiveBanners(asset: Asset?, isGlobal: Boolean): List<Banner>
}
