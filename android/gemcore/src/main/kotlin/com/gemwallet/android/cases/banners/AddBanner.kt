package com.gemwallet.android.cases.banners

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId

interface AddBanner {
    suspend fun addBanner(
        walletId: WalletId? = null,
        asset: Asset? = null,
        chain: Chain?,
        event: BannerEvent,
        state: BannerState = BannerState.Active,
    )
}