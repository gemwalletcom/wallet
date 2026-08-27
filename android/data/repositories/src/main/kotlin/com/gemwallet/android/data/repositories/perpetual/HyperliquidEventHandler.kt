package com.gemwallet.android.data.repositories.perpetual

import android.util.Log
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.domains.perpetual.toGem
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartCandleUpdate
import com.wallet.core.primitives.PerpetualAccountMode
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import uniffi.gemstone.GemHyperliquidOpenOrder
import uniffi.gemstone.GemHyperliquidSocketMessage
import uniffi.gemstone.PerpetualBalance as GemPerpetualBalance
import uniffi.gemstone.PerpetualPosition as GemPerpetualPosition
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.Hyperliquid

class HyperliquidEventHandler(
    private val perpetualService: GemPerpetualService,
    private val hyperliquid: Hyperliquid,
) {
    private val chartFlow = MutableSharedFlow<ChartCandleUpdate>(
        extraBufferCapacity = CHART_BUFFER_CAPACITY,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )
    val chartUpdates: Flow<ChartCandleUpdate> = chartFlow.asSharedFlow()

    suspend fun handle(walletId: WalletId, mode: PerpetualAccountMode, text: String) {
        runCatchingCancellable {
            when (val message = hyperliquid.parseWebsocketData(text.encodeToByteArray(), mode.toGem())) {
                is GemHyperliquidSocketMessage.AccountState -> handleAccountState(walletId, message.balance, message.positions)
                is GemHyperliquidSocketMessage.SpotState -> perpetualService.updateBalance(walletId.id, message.balance)
                is GemHyperliquidSocketMessage.OpenOrders -> handleOpenOrders(walletId, message.orders)
                is GemHyperliquidSocketMessage.Candle -> chartFlow.emit(message.candle.decodeJson())
                is GemHyperliquidSocketMessage.MarketData -> perpetualService.updateMarket(message.market)
                is GemHyperliquidSocketMessage.MarketPrices -> perpetualService.updatePrices(message.prices)
                is GemHyperliquidSocketMessage.SubscriptionResponse -> Log.d(TAG, "Subscription response: ${message.subscriptionType}")
                is GemHyperliquidSocketMessage.Error -> Log.e(TAG, "Error message: ${message.message}")
                GemHyperliquidSocketMessage.Unknown -> Log.d(TAG, "Unknown message: ${text.take(100)}")
            }
        }.onFailure { Log.e(TAG, "Handle message error: ${text.take(100)}", it) }
    }

    private suspend fun handleAccountState(
        walletId: WalletId,
        balance: GemPerpetualBalance?,
        positions: List<GemPerpetualPosition>,
    ) {
        val diff = hyperliquid.diffClearinghousePositions(positions, hypercorePositions(walletId))
        perpetualService.updatePositions(walletId.id, diff.positions, diff.deletePositionIds)
        balance?.let { perpetualService.updateBalance(walletId.id, it) }
    }

    private suspend fun handleOpenOrders(walletId: WalletId, orders: List<GemHyperliquidOpenOrder>) {
        val diff = hyperliquid.diffOpenOrdersPositions(orders, hypercorePositions(walletId))
        perpetualService.updatePositions(walletId.id, diff.positions, diff.deletePositionIds)
    }

    private suspend fun hypercorePositions(walletId: WalletId): List<GemPerpetualPosition> =
        perpetualService.getPositions(walletId.id, Chain.HyperCore.string)

    companion object {
        private const val TAG = "HyperliquidEventHandler"
        private const val CHART_BUFFER_CAPACITY = 64
    }
}
