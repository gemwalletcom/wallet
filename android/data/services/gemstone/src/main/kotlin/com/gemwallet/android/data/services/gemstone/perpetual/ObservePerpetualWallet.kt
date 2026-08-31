package com.gemwallet.android.data.services.gemstone.perpetual

import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemPreferencesService
import javax.inject.Inject

class ObservePerpetualWallet @Inject constructor(
    private val getCurrentWallet: GetCurrentWallet,
    private val userConfig: UserConfig,
    private val preferencesService: GemPreferencesService,
) {
    operator fun invoke(): Flow<Wallet?> = combine(
        getCurrentWallet.observe(),
        userConfig.isPerpetualEnabled(),
    ) { wallet, _ ->
        wallet?.takeIf { preferencesService.showPerpetuals(it.toJson()) }
    }.distinctUntilChanged()
}
