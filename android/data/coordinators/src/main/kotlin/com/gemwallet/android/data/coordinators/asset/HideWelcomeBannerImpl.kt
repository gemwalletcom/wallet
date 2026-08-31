package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.HideWelcomeBanner
import com.gemwallet.android.application.banner.cases.ApplyBannerAction
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.banner.BannerAction
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState

class HideWelcomeBannerImpl(
    private val getSession: GetSession,
    private val applyBannerAction: ApplyBannerAction,
) : HideWelcomeBanner {

    override suspend fun invoke() {
        val wallet = getSession().value?.wallet ?: return
        applyBannerAction(
            Banner(walletId = wallet.id, asset = null, event = BannerEvent.Onboarding, state = BannerState.Active),
            BannerAction.Close,
        )
    }
}
