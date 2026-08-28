package com.gemwallet.android.data.repositories.perpetual

import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.session.currentWallet
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemPreferencesService
import javax.inject.Inject

class ObservePerpetualWallet @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val userConfig: UserConfig,
    private val preferencesService: GemPreferencesService,
) {
    operator fun invoke(): Flow<Wallet?> = combine(
        sessionRepository.currentWallet(),
        userConfig.isPerpetualEnabled(),
    ) { wallet, isEnabled ->
        wallet?.takeIf { isEnabled && preferencesService.showPerpetuals(it.toJson()) }
    }.distinctUntilChanged()
}
