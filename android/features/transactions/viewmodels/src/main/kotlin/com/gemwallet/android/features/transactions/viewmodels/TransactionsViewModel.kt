package com.gemwallet.android.features.transactions.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.connection.cases.ObserveRefreshInterval
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.transactions
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemListWindow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.GemTransactionsFilterSession
import uniffi.gemstone.GemTransactionsFilterView
import uniffi.gemstone.GemTransactionsServiceInterface
import uniffi.gemstone.listPhase
import uniffi.gemstone.newTransactionsFilterSession
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class TransactionsViewModel @Inject constructor(
    getSession: GetSession,
    getTransactions: GetTransactions,
    private val service: GemTransactionsServiceInterface,
    private val observeRefreshInterval: ObserveRefreshInterval,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    val refreshIntervalMillis: StateFlow<Long> = observeRefreshInterval.refreshIntervalMillis(GemRefreshKind.WALLET)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing

    private val list = MutableStateFlow(ActivityList(newTransactionsFilterSession(emptyList(), WalletType.Multicoin.toGem())))

    val filter: StateFlow<GemTransactionsFilterSession> = list.map { it.session }
        .stateIn(viewModelScope, SharingStarted.Eagerly, list.value.session)

    val filterView: StateFlow<GemTransactionsFilterView> = list.map { it.view }
        .stateIn(viewModelScope, SharingStarted.Eagerly, list.value.view)

    val session = getSession()
        .stateIn(viewModelScope, started = SharingStarted.Eagerly, null)

    val walletId: StateFlow<WalletId?> = session
        .map { it?.wallet?.id }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private var syncedWalletId: WalletId? = null
    private var refreshJob: Job? = null

    val transactions = list.map { it.view.filter.toPrimitives() to it.window.limit }
        .distinctUntilChanged()
        .flatMapLatest { (filter, limit) -> getTransactions.getTransactions(filter, limit) }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.Eagerly,
            initialValue = getTransactions.stored(list.value.view.filter.toPrimitives(), list.value.window.limit).takeIf { it.isNotEmpty() },
        )

    private val transactionsState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    val phase: StateFlow<GemListPhase?> = combine(transactionsState, transactions, filterView, ::phase)
        .stateIn(viewModelScope, SharingStarted.Eagerly, phase(transactionsState.value, transactions.value, filterView.value))

    init {
        viewModelScope.launch {
            session
                .filterNotNull()
                .distinctUntilChangedBy { it.wallet.id }
                .collect { list.value = ActivityList(newTransactionsFilterSession(service.filterChains(it.wallet.toGem()), it.wallet.type.toGem())) }
        }
    }

    fun syncIfNeeded(): Job? {
        val current = walletId.value ?: return null
        if (current == syncedWalletId) return null
        syncedWalletId = current
        transactionsState.value = GemLoadState.Loading
        return sync(current, showsSpinner = false)
    }

    fun refresh(): Job? {
        val current = walletId.value ?: return null
        return sync(current, showsSpinner = true)
    }

    private fun sync(wallet: WalletId, showsSpinner: Boolean): Job {
        refreshJob?.cancel()
        val job = viewModelScope.launch(ioDispatcher, start = CoroutineStart.LAZY) {
            if (showsSpinner) _isRefreshing.update { true }
            try {
                val state = service.refresh(null)
                if (walletId.value != wallet) return@launch
                transactionsState.value = state
                if (state is GemLoadState.Error && syncedWalletId == wallet) {
                    syncedWalletId = null
                }
            } finally {
                if (refreshJob == coroutineContext[Job]) _isRefreshing.update { false }
            }
        }
        refreshJob = job
        job.start()
        return job
    }

    private fun phase(state: GemLoadState, rows: List<GemTransactionRow>?, view: GemTransactionsFilterView): GemListPhase? = rows?.let { listPhase(state, it.isNotEmpty(), view.emptyState) }

    fun setChainsFilter(chains: List<Chain>) {
        updateFilter { it.onChains(chains.map { chain -> chain.string }) }
    }

    fun setTypesFilter(types: List<GemTransactionFilter>) {
        updateFilter { it.onTypes(types) }
    }

    fun clearFilters() {
        updateFilter { it.onClear() }
    }

    fun onEndReached() {
        list.update { it.copy(window = it.window.onEndReached(transactions.value.orEmpty().size)) }
    }

    private fun updateFilter(change: (GemTransactionsFilterSession) -> GemTransactionsFilterSession) {
        list.update { current -> change(current.session).takeIf { it != current.session }?.let(::ActivityList) ?: current }
    }
}

private data class ActivityList(val session: GemTransactionsFilterSession, val window: GemListWindow = GemListWindow.transactions) {
    val view: GemTransactionsFilterView = session.viewState()
}
