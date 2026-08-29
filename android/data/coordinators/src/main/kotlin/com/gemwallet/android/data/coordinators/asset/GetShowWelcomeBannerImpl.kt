package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetShowWelcomeBanner
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.gemstone.GemstoneBannerStore
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemBannerService

@OptIn(ExperimentalCoroutinesApi::class)
class GetShowWelcomeBannerImpl(
    private val sessionRepository: SessionRepository,
    private val bannerStore: GemstoneBannerStore,
    private val bannerService: GemBannerService,
    private val getActiveAssetsInfo: GetActiveAssetsInfo,
) : GetShowWelcomeBanner {

    override fun invoke(): Flow<Boolean> {
        return sessionRepository.session()
            .filterNotNull()
            .flatMapLatest { session ->
                val isWalletEmpty = getActiveAssetsInfo
                    .getAssetsInfo(hideBalance = false)
                    .map { items -> items.all { it.isZeroBalance } }
                combine(isWalletEmpty, bannerStore.observeBanner(onboardingBannerKey(session.wallet))) { isEmpty, banner ->
                    banner != null && bannerService.showsOnboarding(banner.state.toJson(), isEmpty)
                }
            }
    }
}

internal fun onboardingBannerKey(wallet: Wallet): GemBannerKey =
    GemBannerKey(walletId = wallet.id.id, assetId = null, event = BannerEvent.Onboarding.toJson())
