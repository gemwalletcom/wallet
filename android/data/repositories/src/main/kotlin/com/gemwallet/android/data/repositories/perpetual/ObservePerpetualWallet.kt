package com.gemwallet.android.data.repositories.perpetual

import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.session.currentWallet
import com.gemwallet.android.ext.hasPerpetualsSupport
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import javax.inject.Inject

class ObservePerpetualWallet @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val userConfig: UserConfig,
) {
    operator fun invoke(): Flow<Wallet?> = combine(
        sessionRepository.currentWallet(),
        userConfig.isPerpetualEnabled(),
    ) { wallet, isEnabled ->
        wallet?.takeIf { isEnabled && it.hasPerpetualsSupport }
    }.distinctUntilChanged()
}
