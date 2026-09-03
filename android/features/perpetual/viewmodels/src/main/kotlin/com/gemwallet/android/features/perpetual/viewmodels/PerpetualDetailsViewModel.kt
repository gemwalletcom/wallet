package com.gemwallet.android.features.perpetual.viewmodels

import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.cases.BuildPerpetualParams
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPosition
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.ChartCandleStick
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
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
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemPerpetualDetailsServiceInterface
import uniffi.gemstone.GemPerpetualPositionKind
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PerpetualDetailsViewModel @Inject constructor(
    private val getPerpetual: GetPerpetual,
    private val getPerpetualPosition: GetPerpetualPosition,
    private val getTransactions: GetTransactions,
    private val buildPerpetualParams: BuildPerpetualParams,
    private val perpetualObserver: PerpetualObserver,
    private val service: GemPerpetualDetailsServiceInterface,
    private val getSession: GetSession,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private companion object {
        const val SubscriptionGraceMillis = 5_000L
        const val TAG = "PerpetualDetails"
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
        runCatchingCancellable { service.syncTransactions(assetId.toIdentifier()) }
        emit(Unit)
    }
        .onStart { emit(Unit) }
        .flowOn(Dispatchers.IO)

    val perpetual = getPerpetual.getPerpetualByAssetId(assetId)
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val position = combine(
        perpetual,
        getSession().filterNotNull(),
    ) { perpetual, session -> perpetual to session.wallet.id }
        .flatMapLatest { (perpetual, walletId) ->
            perpetual?.let { getPerpetualPosition.getPositionByPerpetual(walletId, it.id) } ?: flowOf(null)
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val transactions = combine(
        getTransactions.getTransactions(transactionFilters),
        transactionSync,
    ) { transactions, _ -> transactions }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val period = MutableStateFlow(service.chartPeriod().decodeJson<ChartPeriod>())

    private val refreshTrigger = MutableStateFlow(0L)
    private val refreshState = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = refreshState.asStateFlow()

    val chart: StateFlow<StateViewType<List<ChartCandleStick>>> = combine(period, refreshTrigger) { period, _ -> period }
        .flatMapLatest { period ->
            flow {
                emit(StateViewType.Loading)
                try {
                    val market = perpetual.value?.perpetual
                    var candles = market?.let { service.candlesticks(it.toJson(), period.toJson()).map { candle -> candle.decodeJson<ChartCandleStick>() } }.orEmpty()
                    refreshState.value = false
                    emit(candles.toChartState())
                    if (market == null) return@flow
                    perpetualObserver.chartUpdates
                        .collect { update ->
                            candles = service.applyCandleUpdate(candles.map { it.toJson() }, update.toJson(), market.toJson(), period.toJson())
                                ?.map { it.decodeJson<ChartCandleStick>() } ?: return@collect
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
                perpetual.map { it?.perpetual }.distinctUntilChanged(),
                period,
            ) { isVisible, market, period ->
                if (isVisible && market != null) market to period else null
            }
                .distinctUntilChanged()
                .collectLatest { subscriptionKey ->
                    val (market, period) = subscriptionKey ?: return@collectLatest
                    val subscriptions = listOf(
                        service.candleSubscription(market.toJson(), period.toJson()),
                        service.marketSubscription(market.toJson()),
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
        viewModelScope.launch(Dispatchers.IO) { service.setChartPeriod(period.toJson()) }
        this.period.update { period }
    }

    fun fetch() {
        refreshTrigger.update { it + 1 }
        viewModelScope.launch(Dispatchers.IO) {
            runCatchingCancellable { service.syncPositions() }
                .onFailure { Log.e(TAG, "perpetual positions sync failed", it) }
        }
    }

    fun refresh() {
        refreshState.value = true
        fetch()
    }

    fun openPosition(direction: PerpetualDirection, amountAction: AmountTransactionAction) =
        position(GemPerpetualPositionKind.Open(direction.toJson()), amountAction)

    fun increasePosition(amountAction: AmountTransactionAction) = position(GemPerpetualPositionKind.Increase, amountAction)

    fun reducePosition(amountAction: AmountTransactionAction) = position(GemPerpetualPositionKind.Reduce, amountAction)

    private fun position(kind: GemPerpetualPositionKind, amountAction: AmountTransactionAction) {
        val perpetualId = perpetual.value?.id ?: return
        viewModelScope.launch {
            runCatchingCancellable { buildPerpetualParams.position(perpetualId, kind) }
                .onSuccess { params -> params?.let(amountAction::invoke) }
                .onFailure { Log.e(TAG, "perpetual position action failed", it) }
        }
    }

    fun closePosition(confirmAction: ConfirmTransactionAction) {
        val perpetualId = perpetual.value?.id ?: return
        viewModelScope.launch {
            runCatchingCancellable { buildPerpetualParams.close(perpetualId) }
                .onSuccess { input -> input?.let(confirmAction::invoke) }
                .onFailure { Log.e(TAG, "perpetual close failed", it) }
        }
    }
}

private fun List<ChartCandleStick>.toChartState(): StateViewType<List<ChartCandleStick>> =
    if (isEmpty()) StateViewType.NoData else StateViewType.Data(this)
