package com.gemwallet.android.data.coordinators.banner

import com.gemwallet.android.application.banner.coordinators.GetActiveBanners
import com.gemwallet.android.cases.banners.GetBannersCase
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import kotlinx.coroutines.flow.firstOrNull

class GetActiveBannersImpl(
    private val sessionRepository: SessionRepository,
    private val getBannersCase: GetBannersCase,
) : GetActiveBanners {

    override suspend fun invoke(asset: Asset?, isGlobal: Boolean): List<Banner> {
        val wallet = if (isGlobal) {
            null
        } else {
            sessionRepository.session().firstOrNull()?.wallet
        }
        return getBannersCase.getActiveBanners(wallet, asset)
    }
}
