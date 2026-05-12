package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.coordinators.GetShowWelcomeBanner
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.WalletSource
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map

@OptIn(ExperimentalCoroutinesApi::class)
class GetShowWelcomeBannerImpl(
    private val sessionRepository: SessionRepository,
    private val userConfig: UserConfig,
    private val getActiveAssetsInfo: GetActiveAssetsInfo,
) : GetShowWelcomeBanner {

    override fun invoke(): Flow<Boolean> {
        return sessionRepository.session()
            .filterNotNull()
            .flatMapLatest { session ->
                val isWalletEmpty = getActiveAssetsInfo
                    .getAssetsInfo(hideBalance = false)
                    .map { items -> items.all { it.isZeroBalance } }
                combine(isWalletEmpty, userConfig.isWelcomeBannerHidden(session.wallet.id.id)) { isEmpty, isHidden ->
                    val created = session.wallet.source == WalletSource.Create
                    isEmpty && created && !isHidden
                }
            }
    }
}
