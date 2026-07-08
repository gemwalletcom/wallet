package com.gemwallet.android.features.perpetual.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.coordinators.BuildPerpetualParams
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetual
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualChartData
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualChartPeriod
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualPosition
import com.gemwallet.android.application.perpetual.coordinators.PerpetualObserver

import com.gemwallet.android.application.perpetual.coordinators.SetPerpetualChartPeriod
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.application.transactions.coordinators.GetTransactions
import com.gemwallet.android.application.transactions.coordinators.SyncAssetTransactions
import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TransactionType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemPerpetualSubscription
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PerpetualDetailsViewModel @Inject constructor(
    private val getPerpetual: GetPerpetual,
    private val getPerpetualPosition: GetPerpetualPosition,
    private val getPerpetualChartData: GetPerpetualChartData,
    private val getTransactions: GetTransactions,
    private val syncAssetTransactions: SyncAssetTransactions,
    private val syncPerpetualPositions: SyncPerpetualPositions,
    private val buildPerpetualParams: BuildPerpetualParams,
    private val perpetualObserver: PerpetualObserver,
    getPerpetualChartPeriod: GetPerpetualChartPeriod,
    private val setPerpetualChartPeriod: SetPerpetualChartPeriod,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private companion object {
        const val SubscriptionGraceMillis = 5_000L
    }

    val assetId = savedStateHandle.requireAssetId()

    private val transactionFilters = listOf(
        TransactionsRequestFilter.Asset(assetId),
        TransactionsRequestFilter.Types(
            listOf(
                TransactionType.PerpetualOpenPosition,
                TransactionType.PerpetualClosePosition,
            )
        )
    )

    private val transactionSync = flow {
        syncAssetTransactions.syncAssetTransactions(assetId)
        emit(Unit)
    }
        .onStart { emit(Unit) }
        .flowOn(Dispatchers.IO)

    val perpetual = getPerpetual.getPerpetualByAssetId(assetId)
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val position = perpetual
        .flatMapLatest { perpetual ->
            perpetual?.let { getPerpetualPosition.getPositionByPerpetual(it.id) } ?: flowOf(null)
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val transactions = combine(
        getTransactions.getTransactions(transactionFilters),
        transactionSync,
    ) { transactions, _ -> transactions }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val period = MutableStateFlow(getPerpetualChartPeriod())

    private val refreshTrigger = MutableStateFlow(0L)
    private val refreshState = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = refreshState.asStateFlow()

    val chart: StateFlow<StateViewType<List<ChartCandleStick>>> = combine(period, refreshTrigger) { period, _ -> period }
        .flatMapLatest { period ->
            flow {
                emit(StateViewType.Loading)
                try {
                    var candles = getPerpetualChartData.getPerpetualChartData(assetId, period)
                    refreshState.value = false
                    emit(candles.toChartState())
                    perpetualObserver.chartUpdates
                        .filter { it.coin == perpetual.value?.coin && it.interval == period.hyperliquidInterval }
                        .collect { update ->
                            candles = candles.mergeCandle(update.candle)
                            emit(candles.toChartState())
                        }
                } catch (e: Exception) {
                    currentCoroutineContext().ensureActive()
                    refreshState.value = false
                    emit(StateViewType.Error)
                }
            }
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(SubscriptionGraceMillis), StateViewType.Loading)

    private val screenVisible = MutableStateFlow(false)

    init {
        viewModelScope.launch {
            combine(
                screenVisible,
                perpetual.map { it?.coin }.distinctUntilChanged(),
                period,
            ) { isVisible, coin, period ->
                if (isVisible && coin != null) coin to period else null
            }
                .distinctUntilChanged()
                .collectLatest { subscriptionKey ->
                    val (coin, period) = subscriptionKey ?: return@collectLatest
                    val subscriptions = listOf(
                        GemPerpetualSubscription.Candle(symbol = coin, interval = period.hyperliquidInterval),
                        GemPerpetualSubscription.MarketData(symbol = coin),
                    )
                    subscriptions.forEach(perpetualObserver::subscribe)
                    try {
                        awaitCancellation()
                    } finally {
                        subscriptions.forEach(perpetualObserver::unsubscribe)
                    }
                }
        }
    }

    fun onScreenEnter() {
        screenVisible.value = true
    }

    fun onScreenExit() {
        screenVisible.value = false
    }

    fun period(period: ChartPeriod) {
        setPerpetualChartPeriod(period)
        this.period.update { period }
    }

    fun fetch() {
        refreshTrigger.update { it + 1 }
        viewModelScope.launch(Dispatchers.IO) {
            syncPerpetualPositions.syncPerpetualPositions()
        }
    }

    fun refresh() {
        refreshState.value = true
        fetch()
    }

    fun openPosition(direction: PerpetualDirection, amountAction: AmountTransactionAction) {
        val perpetualId = perpetual.value?.id ?: return
        viewModelScope.launch {
            buildPerpetualParams.open(perpetualId, direction)?.let(amountAction::invoke)
        }
    }

    fun increasePosition(amountAction: AmountTransactionAction) {
        val perpetualId = perpetual.value?.id ?: return
        viewModelScope.launch {
            buildPerpetualParams.increase(perpetualId)?.let(amountAction::invoke)
        }
    }

    fun reducePosition(amountAction: AmountTransactionAction) {
        val perpetualId = perpetual.value?.id ?: return
        viewModelScope.launch {
            buildPerpetualParams.reduce(perpetualId)?.let(amountAction::invoke)
        }
    }

    fun closePosition(confirmAction: ConfirmTransactionAction) {
        val perpetualId = perpetual.value?.id ?: return
        viewModelScope.launch {
            buildPerpetualParams.close(perpetualId)?.let(confirmAction::invoke)
        }
    }
}

private fun List<ChartCandleStick>.toChartState(): StateViewType<List<ChartCandleStick>> =
    if (isEmpty()) StateViewType.NoData else StateViewType.Data(this)
