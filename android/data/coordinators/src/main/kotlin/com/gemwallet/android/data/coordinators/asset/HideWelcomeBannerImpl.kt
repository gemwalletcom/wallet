package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.HideWelcomeBanner
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository

class HideWelcomeBannerImpl(
    private val sessionRepository: SessionRepository,
    private val userConfig: UserConfig,
) : HideWelcomeBanner {

    override suspend fun invoke() {
        val walletId = sessionRepository.session().value?.wallet?.id ?: return
        userConfig.hideWelcomeBanner(walletId)
    }
}
