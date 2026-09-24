package com.gemwallet.android.application.banner.cases

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import kotlinx.coroutines.flow.Flow

interface GetAssetBanners {
    operator fun invoke(asset: Asset): Flow<List<Banner>>
}
