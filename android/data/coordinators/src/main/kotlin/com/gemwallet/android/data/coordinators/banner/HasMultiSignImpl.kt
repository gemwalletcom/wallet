package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.cases.banners.HasMultiSign
import com.gemwallet.android.data.repositories.gemstone.GemstoneBannerStore
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow

class HasMultiSignImpl(
    private val bannerStore: GemstoneBannerStore,
) : HasMultiSign {

    override fun hasMultiSign(wallet: Wallet): Flow<Boolean> = bannerStore.observeMultiSign(wallet.id.id)
}
