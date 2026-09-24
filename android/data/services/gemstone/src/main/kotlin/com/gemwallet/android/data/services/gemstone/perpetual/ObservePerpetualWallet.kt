package com.gemwallet.android.data.services.gemstone.perpetual

import android.util.Log
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import uniffi.gemstone.GemPerpetualEnablementTrigger
import uniffi.gemstone.GemPerpetualServiceInterface
import javax.inject.Inject

class ObservePerpetualWallet @Inject constructor(
    private val getCurrentWallet: GetCurrentWallet,
    private val userConfig: UserConfig,
    private val perpetualService: GemPerpetualServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) {
    operator fun invoke(): Flow<Wallet?> = combine(
        getCurrentWallet.observe(),
        userConfig.isPerpetualEnabled(),
    ) { wallet, _ ->
        val connects = runCatchingCancellable { perpetualService.syncEnablement(wallet?.toGem(), GemPerpetualEnablementTrigger.WALLET_CHANGED) }
            .onFailure { Log.e(TAG, "perpetual enablement failed", it) }
            .getOrDefault(false)
        wallet?.takeIf { connects }
    }.distinctUntilChanged().flowOn(ioDispatcher)

    private companion object {
        const val TAG = "ObservePerpetualWallet"
    }
}
