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
import com.gemwallet.android.ext.GemConstants
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
import com.gemwallet.android.ui.localization.text
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
import kotlinx.coroutines.flow.MutableSharedFlow
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
import kotlinx.coroutines.flow.transformLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPerpetualDetails
import uniffi.gemstone.GemPerpetualDetailsServiceInterface
import uniffi.gemstone.GemPerpetualPositionKind
import uniffi.gemstone.candleSession
import uniffi.gemstone.candleTooltip
import uniffi.gemstone.loadError
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
        TransactionsRequestFilter.Types(GemConstants.perpetualActivityTypes),
    )

    private val storedRefreshRequests = MutableSharedFlow<Unit>(extraBufferCapacity = 1)

    private val storedSync = storedRefreshRequests
        .transformLatest { _ ->
            runCatchingCancellable { service.refresh(assetId.toIdentifier()) }
                .getOrNull()
                ?.forEach { Log.e(TAG, "perpetual refresh failed at ${it.step}: ${it.message}") }
            emit(Unit)
        }
        .onStart { emit(Unit) }
        .flowOn(ioDispatcher)

    val perpetual = getPerpetual.getPerpetualByAssetId(assetId)
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val position = combine(
        perpetual.map { it?.perpetual?.id }.distinctUntilChanged(),
        getSession().filterNotNull().map { it.wallet.id }.distinctUntilChanged(),
        ::Pair,
    )
        .flatMapLatest { (perpetualId, walletId) ->
            perpetualId?.let { getPerpetualPosition.getPositionByPerpetual(walletId, it) } ?: flowOf(null)
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val detailsState: StateFlow<GemPerpetualDetails?> = combine(perpetual, position) { perpetual, position ->
        perpetual?.let { service.details(it.perpetual.toGem(), it.asset.toGem(), listOfNotNull(position?.position?.toGem())) }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val positionListItem: StateFlow<ListItemModel?> = detailsState.map { it?.positionRow?.listItem(context) }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(SubscriptionGraceMillis), null)

    val details: StateFlow<PerpetualDetailsUIModel?> = detailsState.map { it?.uiModel(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val transactions = combine(
        getTransactions.getTransactions(transactionFilters),
        storedSync,
    ) { transactions, _ -> transactions }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, getTransactions.stored(transactionFilters))

    private val candles = MutableStateFlow(candleSession(service.chartPeriod()))

    val period: StateFlow<ChartPeriod> = candles.map { it.period.toPrimitives() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, candles.value.period.toPrimitives())

    private val candleViewState = candles.map { it.viewState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, candles.value.viewState())

    val isRefreshing: StateFlow<Boolean> = candleViewState.map { it.isRefreshing }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val chart: StateFlow<StateViewType<PerpetualChartUIModel>> = combine(candleViewState, position) { state, position ->
        when (val error = loadError(state.state, state.candles.isNotEmpty())) {
            null -> when (state.state) {
                GemLoadState.Loading -> StateViewType.Loading
                GemLoadState.NoData -> StateViewType.NoData
                else -> StateViewType.Data(PerpetualChartUIModel.from(state.candles.map { it.toPrimitives() }, position?.position, context))
            }

            else -> StateViewType.Error(error.errorText().text(context))
        }
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(SubscriptionGraceMillis), StateViewType.Loading)

    fun tooltip(candle: ChartCandleStick): CandlestickTooltipUIModel = candleTooltip(candle.toGem()).uiModel(context)

    private val screenVisible = MutableStateFlow(false)

    init {
        viewModelScope.launch {
            combine(perpetual.map { it?.perpetual }.distinctUntilChanged(), candles, ::Pair).collectLatest { (market, session) ->
                val selected = market?.let { session.onSelectMarket(it.toGem()) } ?: return@collectLatest
                if (selected != session) {
                    candles.value = selected
                    return@collectLatest
                }
                val request = session.request()?.takeIf { session.needsCandles() } ?: return@collectLatest
                val result = withContext(ioDispatcher) { service.candles(request) }
                candles.update { it.onResult(result) }
            }
        }
        viewModelScope.launch {
            perpetualObserver.chartUpdates.collect { update ->
                val market = perpetual.value?.perpetual ?: return@collect
                val session = candles.value
                val merged = withContext(ioDispatcher) { service.mergedCandles(session.candles, update.toGem(), market.toGem(), session.period) } ?: return@collect
                candles.update { it.onCandles(merged) }
            }
        }
        viewModelScope.launch {
            combine(
                screenVisible,
                perpetual.map { it?.perpetual },
                period,
            ) { isVisible, market, period ->
                market?.takeIf { isVisible }?.let {
                    listOf(
                        service.candleSubscription(it.toGem(), period.toGem()),
                        service.marketSubscription(it.toGem()),
                    )
                }
            }
                .distinctUntilChanged()
                .collectLatest { subscriptions ->
                    subscriptions ?: return@collectLatest
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
        candles.update { it.onSelectPeriod(period.toGem()) }
    }

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    fun refreshPerpetual() {
        storedRefreshRequests.tryEmit(Unit)
    }

    fun refresh() {
        candles.update { it.onRefresh() }
        refreshPerpetual()
    }

    fun openPosition(direction: PerpetualDirection, amountAction: AmountTransactionAction) = position(GemPerpetualPositionKind.Open(direction.toGem()), amountAction)

    fun increasePosition(amountAction: AmountTransactionAction) = position(GemPerpetualPositionKind.Increase, amountAction)

    fun reducePosition(amountAction: AmountTransactionAction) = position(GemPerpetualPositionKind.Reduce, amountAction)

    private fun position(kind: GemPerpetualPositionKind, amountAction: AmountTransactionAction) {
        val data = perpetual.value ?: return
        runCatching { service.positionAction(data.perpetual.toGem(), data.asset.toGem(), details.value?.position, kind) }
            .onSuccess { action -> amountAction(AmountParams.Perpetual(assetId = data.asset.id, perpetualId = data.perpetual.id, positionAction = action)) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun closePosition(confirmAction: ConfirmTransactionAction) {
        val data = perpetual.value ?: return
        runCatching { service.closeTransfer(data.perpetual.toGem(), data.asset.toGem(), details.value?.position) }
            .onSuccess { confirmAction(ConfirmTransferInput(it)) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun clearError() = errorState.update { null }
}
