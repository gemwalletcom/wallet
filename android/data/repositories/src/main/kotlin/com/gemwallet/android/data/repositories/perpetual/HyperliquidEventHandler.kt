package com.gemwallet.android.data.repositories.perpetual

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.domains.perpetual.toGem
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.ChartCandleUpdate
import com.wallet.core.primitives.PerpetualAccountMode
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPerpetualSocketUpdate

class HyperliquidEventHandler(
    private val perpetualService: GemPerpetualService,
) {
    private val chartFlow = MutableSharedFlow<ChartCandleUpdate>(
        extraBufferCapacity = CHART_BUFFER_CAPACITY,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    val chartUpdates: Flow<ChartCandleUpdate> = chartFlow.asSharedFlow()

    suspend fun handle(walletId: WalletId, mode: PerpetualAccountMode, text: String) {
        runCatchingCancellable {
            when (val update = perpetualService.applySocketMessage(walletId.id, mode.toGem(), text.encodeToByteArray())) {
                GemPerpetualSocketUpdate.Applied -> Unit
                is GemPerpetualSocketUpdate.Candle -> chartFlow.emit(update.candle.decodeJson())
                is GemPerpetualSocketUpdate.SubscriptionResponse -> Log.d(TAG, "Subscription response: ${update.subscriptionType}")
                is GemPerpetualSocketUpdate.Error -> Log.e(TAG, "Error message: ${update.message}")
                GemPerpetualSocketUpdate.Unknown -> Log.d(TAG, "Unknown message: ${text.take(100)}")
            }
        }.onFailure { Log.e(TAG, "Handle message error: ${text.take(100)}", it) }
    }

    companion object {
        private const val TAG = "HyperliquidEventHandler"
        private const val CHART_BUFFER_CAPACITY = 64
    }
}
