package com.gemwallet.android.application.perpetual.coordinators

import com.wallet.core.primitives.ChartCandleUpdate
import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemPerpetualSubscription

interface PerpetualObserver {
    val chartUpdates: Flow<ChartCandleUpdate>

    fun subscribe(subscription: GemPerpetualSubscription)

    fun unsubscribe(subscription: GemPerpetualSubscription)
}
