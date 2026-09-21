package com.gemwallet.android.application.banner.cases

import com.wallet.core.primitives.Asset
import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemBannerRow

interface GetActiveBanners {
    operator fun invoke(asset: Asset): Flow<List<GemBannerRow>>
}
