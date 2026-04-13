package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetShowWelcomeBanner
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.WalletSource
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetShowWelcomeBannerImpl(
    private val sessionRepository: SessionRepository,
    private val userConfig: UserConfig,
) : GetShowWelcomeBanner {

    override fun invoke(isWalletEmpty: Flow<Boolean>): Flow<Boolean> {
        return sessionRepository.session()
            .filterNotNull()
            .flatMapLatest { session ->
                combine(isWalletEmpty, userConfig.isWelcomeBannerHidden(session.wallet.id)) { isEmpty, isHidden ->
                    val created = session.wallet.source == WalletSource.Create
                    isEmpty && created && !isHidden
                }
            }
    }
}
