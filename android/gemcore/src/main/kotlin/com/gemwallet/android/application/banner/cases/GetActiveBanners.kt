package com.gemwallet.android.application.banner.cases

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import kotlinx.coroutines.flow.Flow

interface GetActiveBanners {
    operator fun invoke(asset: Asset?, isGlobal: Boolean): Flow<List<Banner>>
}
