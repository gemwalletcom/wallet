package com.gemwallet.android.application.banner.cases

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner

interface GetActiveBanners {
    suspend operator fun invoke(asset: Asset?, isGlobal: Boolean): List<Banner>
}
