package com.gemwallet.android.features.perpetual.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPosition
import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.perpetual.viewmodels.model.PerpetualDetailsUIModel
import com.gemwallet.android.features.perpetual.viewmodels.model.uiModel
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualChartUIModel
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.components.chart.CandlestickTooltipUIModel
import com.gemwallet.android.ui.components.chart.uiModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.perpetual.listItem
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.flatMap
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PerpetualDirection
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
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
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemPerpetualDetailsServiceInterface
import uniffi.gemstone.GemPerpetualPositionKind
import uniffi.gemstone.candleTooltip
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PerpetualDetailsViewModel @Inject constructor(
    private val getPerpetual: GetPerpetual,
    private val getPerpetualPosition: GetPerpetualPosition,
    private val getTransactions: GetTransactions,
    private val perpetualObserver: PerpetualObserver,
    private val service: GemPerpetualDetailsServiceInterface,
    private val getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private companion object {
        const val SubscriptionGraceMillis = 5_000L
        const val TAG = "PerpetualDetails"
    }

    val assetId = savedStateHandle.requireAssetId()

    private val transactionFilters = listOf(
        TransactionsRequestFilter.Asset(assetId),
        TransactionsRequestFilter.Types(service.activityTypes().map { it.toPrimitives() }),
    )

    private val transactionSync = flow {
        runCatchingCancellable { service.syncTransactions(assetId.toIdentifier()) }
        emit(Unit)
    }
        .onStart { emit(Unit) }
        .flowOn(ioDispatcher)

    val perpetual = getPerpetual.getPerpetualByAssetId(assetId)
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val position = combine(
        perpetual,
        getSession().filterNotNull(),
    ) { perpetual, session -> perpetual to session.wallet.id }
        .flatMapLatest { (perpetual, walletId) ->
            perpetual?.let { getPerpetualPosition.getPositionByPerpetual(walletId, it.id) } ?: flowOf(null)
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val positionListItem: StateFlow<ListItemModel?> = position.map { it?.listItem(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val details: StateFlow<PerpetualDetailsUIModel?> = combine(perpetual, position) { perpetual, position ->
        perpetual?.let { service.details(it.perpetual.toGem(), it.asset.toGem(), listOfNotNull(position?.position?.toGem())).uiModel(context) }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val transactions = combine(
        getTransactions.getTransactions(transactionFilters),
        transactionSync,
    ) { transactions, _ -> transactions }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, getTransactions.stored(transactionFilters))

    val period = MutableStateFlow(service.chartPeriod().toPrimitives())

    private val refreshTrigger = MutableStateFlow(0L)
    private val refreshState = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = refreshState.asStateFlow()

    private val candles: StateFlow<StateViewType<List<ChartCandleStick>>> = combine(period, refreshTrigger) { period, _ -> period }
        .flatMapLatest { period ->
            flow {
                emit(StateViewType.Loading)
                try {
                    val market = perpetual.value?.perpetual
                    var candles = market?.let { service.candlesticks(it.toGem(), period.toGem()).map { candle -> candle.toPrimitives() } }.orEmpty()
                    refreshState.value = false
                    emit(candles.toChartState())
                    if (market == null) return@flow
                    perpetualObserver.chartUpdates
                        .collect { update ->
                            candles = service.mergedCandles(candles.map { it.toGem() }, update.toGem(), market.toGem(), period.toGem())
                                ?.map { it.toPrimitives() } ?: return@collect
                            emit(candles.toChartState())
                        }
                } catch (e: Exception) {
                    currentCoroutineContext().ensureActive()
                    refreshState.value = false
                    emit(StateViewType.Error)
                }
            }
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(SubscriptionGraceMillis), StateViewType.Loading)

    val chart: StateFlow<StateViewType<PerpetualChartUIModel>> = combine(candles, position) { state, position ->
        state.flatMap { StateViewType.Data(PerpetualChartUIModel.from(it, position?.position, context)) }
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(SubscriptionGraceMillis), StateViewType.Loading)

    fun tooltip(candle: ChartCandleStick): CandlestickTooltipUIModel = candleTooltip(candle.toGem()).uiModel(context)

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
                        service.candleSubscription(market.toGem(), period.toGem()),
                        service.marketSubscription(market.toGem()),
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
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.setChartPeriod(period.toGem()) }
                .onFailure { Log.e(TAG, "storing the chart period failed", it) }
        }
        this.period.update { period }
    }

    private val errorState = MutableStateFlow<GemErrorText?>(null)
    val error: StateFlow<GemErrorText?> = errorState.asStateFlow()

    fun fetch() {
        refreshTrigger.update { it + 1 }
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.syncPositions() }
                .onFailure { Log.e(TAG, "perpetual positions sync failed", it) }
        }
    }

    fun refresh() {
        refreshState.value = true
        fetch()
    }

    fun openPosition(direction: PerpetualDirection, amountAction: AmountTransactionAction) = position(GemPerpetualPositionKind.Open(direction.toGem()), amountAction)

    fun increasePosition(amountAction: AmountTransactionAction) = position(GemPerpetualPositionKind.Increase, amountAction)

    fun reducePosition(amountAction: AmountTransactionAction) = position(GemPerpetualPositionKind.Reduce, amountAction)

    private fun position(kind: GemPerpetualPositionKind, amountAction: AmountTransactionAction) {
        val data = perpetual.value ?: return
        val action = service.positionAction(data.perpetual.toGem(), data.asset.toGem(), details.value?.position, kind)
        amountAction(AmountParams.Perpetual(assetId = data.asset.id, perpetualId = data.perpetual.id, positionAction = action))
    }

    fun closePosition(confirmAction: ConfirmTransactionAction) {
        val data = perpetual.value ?: return
        confirmAction(ConfirmTransferInput(service.closeTransfer(data.perpetual.toGem(), data.asset.toGem(), details.value?.position)))
    }

    fun clearError() = errorState.update { null }
}

private fun List<ChartCandleStick>.toChartState(): StateViewType<List<ChartCandleStick>> = if (isEmpty()) StateViewType.NoData else StateViewType.Data(this)
