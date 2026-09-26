package com.gemwallet.android.features.transactions.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.connection.cases.ObserveRefreshInterval
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
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
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.GemTransactionsFilterSession
import uniffi.gemstone.GemTransactionsFilterView
import uniffi.gemstone.GemTransactionsServiceInterface
import uniffi.gemstone.loadError
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

    private val _filter = MutableStateFlow(newTransactionsFilterSession(emptyList(), WalletType.Multicoin.toGem()))
    val filter: StateFlow<GemTransactionsFilterSession> = _filter

    val filterView: StateFlow<GemTransactionsFilterView> = _filter
        .map { it.viewState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, _filter.value.viewState())

    val session = getSession()
        .stateIn(viewModelScope, started = SharingStarted.Eagerly, null)

    val walletId: StateFlow<WalletId?> = session
        .map { it?.wallet?.id }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private var syncedWalletId: WalletId? = null
    private var refreshJob: Job? = null

    val transactions = filterView
        .map { it.filter.toPrimitives() }
        .distinctUntilChanged()
        .flatMapLatest { filter -> getTransactions.getTransactions(filter, GemConstants.transactionsListLimit) }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.Eagerly,
            initialValue = getTransactions.stored(filterView.value.filter.toPrimitives(), GemConstants.transactionsListLimit).takeIf { it.isNotEmpty() },
        )

    private val transactionsState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    val errorRow: StateFlow<GemListRow?> = combine(transactionsState, transactions) { state, items ->
        loadError(state, items?.isEmpty() != true)?.let { GemListRow.Error(it) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    init {
        viewModelScope.launch {
            session
                .filterNotNull()
                .distinctUntilChangedBy { it.wallet.id }
                .collect { _filter.value = newTransactionsFilterSession(service.filterChains(it.wallet.toGem()), it.wallet.type.toGem()) }
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
                val state = service.refresh(null, !transactions.value.isNullOrEmpty())
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

    fun setChainsFilter(chains: List<Chain>) {
        _filter.update { it.onChains(chains.map { chain -> chain.string }) }
    }

    fun setTypesFilter(types: List<GemTransactionFilter>) {
        _filter.update { it.onTypes(types) }
    }

    fun clearFilters() {
        _filter.update { it.onClear() }
    }
}
