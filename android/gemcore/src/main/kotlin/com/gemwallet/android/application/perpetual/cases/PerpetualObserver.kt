package com.gemwallet.android.application.perpetual.cases

import com.wallet.core.primitives.ChartCandleUpdate
import com.wallet.core.primitives.PerpetualAccountMode
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemPerpetualSubscription

interface PerpetualObserver {
    val chartUpdates: Flow<ChartCandleUpdate>

    fun subscribe(subscription: GemPerpetualSubscription)

    fun unsubscribe(subscription: GemPerpetualSubscription)
}
