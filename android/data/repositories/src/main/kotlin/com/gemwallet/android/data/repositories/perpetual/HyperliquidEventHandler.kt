package com.gemwallet.android.data.repositories.perpetual

import android.util.Log
import com.gemwallet.android.blockchain.gemstone.toDTO
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.domains.perpetual.toGem
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.ext.runCatchingCancellable
import com.wallet.core.primitives.ChartCandleUpdate
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import uniffi.gemstone.GemHyperliquidOpenOrder
import uniffi.gemstone.GemHyperliquidSocketMessage
import uniffi.gemstone.GemPerpetualBalance
import uniffi.gemstone.GemPerpetualPosition
import uniffi.gemstone.Hyperliquid

class HyperliquidEventHandler(
    private val perpetualRepository: PerpetualRepository,
    private val hyperliquid: Hyperliquid,
) {

    private val chartFlow = MutableSharedFlow<ChartCandleUpdate>(
        extraBufferCapacity = CHART_BUFFER_CAPACITY,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )
    val chartUpdates: Flow<ChartCandleUpdate> = chartFlow.asSharedFlow()

    private val pricesUpdateIntervalMs = PerpetualConfig.pricesUpdateIntervalSeconds * 1000L
    private var pricesUpdatedAt = 0L

    suspend fun handle(walletId: WalletId, text: String) {
        runCatchingCancellable {
            when (val message = hyperliquid.parseWebsocketData(text.encodeToByteArray())) {
                is GemHyperliquidSocketMessage.AccountState -> handleAccountState(walletId, message.balance, message.positions)
                is GemHyperliquidSocketMessage.OpenOrders -> handleOpenOrders(walletId, message.orders)
                is GemHyperliquidSocketMessage.Candle -> chartFlow.emit(message.candle.toDTO())
                is GemHyperliquidSocketMessage.MarketData -> perpetualRepository.updateMarket(message.market.toDTO())
                is GemHyperliquidSocketMessage.MarketPrices -> handleMarketPrices(message.prices)
                is GemHyperliquidSocketMessage.SubscriptionResponse -> Log.d(TAG, "Subscription response: ${message.subscriptionType}")
                GemHyperliquidSocketMessage.Unknown -> Log.d(TAG, "Unknown message: ${text.take(100)}")
            }
        }.onFailure { Log.e(TAG, "Handle message error: ${text.take(100)}", it) }
    }

    private suspend fun handleAccountState(
        walletId: WalletId,
        balance: GemPerpetualBalance,
        positions: List<GemPerpetualPosition>,
    ) {
        val diff = hyperliquid.diffClearinghousePositions(positions, getProviderPositions(walletId))
        perpetualRepository.putAsset(HypercoreUSDC)
        perpetualRepository.putBalance(walletId, HypercoreUSDC, balance.toDTO())
        perpetualRepository.applyPositionsDiff(walletId, diff.deletePositionIds, diff.positions.mapNotNull { it.toDTO() })
    }

    private suspend fun handleOpenOrders(walletId: WalletId, orders: List<GemHyperliquidOpenOrder>) {
        val diff = hyperliquid.diffOpenOrdersPositions(orders, getProviderPositions(walletId))
        perpetualRepository.applyPositionsDiff(walletId, diff.deletePositionIds, diff.positions.mapNotNull { it.toDTO() })
    }

    private suspend fun handleMarketPrices(prices: Map<String, Double>) {
        val now = System.currentTimeMillis()
        if (now - pricesUpdatedAt < pricesUpdateIntervalMs) return
        pricesUpdatedAt = now
        perpetualRepository.updatePrices(prices)
    }

    private suspend fun getProviderPositions(walletId: WalletId): List<GemPerpetualPosition> {
        return perpetualRepository.getProviderPositions(walletId, PerpetualProvider.Hypercore).map { it.toGem() }
    }

    companion object {
        private const val TAG = "HyperliquidEventHandler"
        private const val CHART_BUFFER_CAPACITY = 64
    }
}
