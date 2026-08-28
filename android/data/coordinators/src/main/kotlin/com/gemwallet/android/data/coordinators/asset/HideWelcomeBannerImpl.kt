package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.HideWelcomeBanner
import com.gemwallet.android.application.banner.coordinators.ApplyBannerAction
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.banner.BannerAction
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState

class HideWelcomeBannerImpl(
    private val sessionRepository: SessionRepository,
    private val applyBannerAction: ApplyBannerAction,
) : HideWelcomeBanner {

    override suspend fun invoke() {
        val wallet = sessionRepository.session().value?.wallet ?: return
        applyBannerAction(
            Banner(walletId = wallet.id, asset = null, event = BannerEvent.Onboarding, state = BannerState.Active),
            BannerAction.Close,
        )
    }
}
