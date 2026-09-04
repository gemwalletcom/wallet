package com.gemwallet.android.data.services.gemstone.perpetual

import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import uniffi.gemstone.GemPerpetualServiceInterface
import javax.inject.Inject

class ObservePerpetualWallet @Inject constructor(
    private val getCurrentWallet: GetCurrentWallet,
    private val userConfig: UserConfig,
    private val perpetualService: GemPerpetualServiceInterface,
) {
    operator fun invoke(): Flow<Wallet?> = combine(
        getCurrentWallet.observe(),
        userConfig.isPerpetualEnabled(),
    ) { wallet, _ ->
        wallet?.takeIf { perpetualService.shouldConnectPerpetuals(it.toJson()) }
    }.distinctUntilChanged()
}
