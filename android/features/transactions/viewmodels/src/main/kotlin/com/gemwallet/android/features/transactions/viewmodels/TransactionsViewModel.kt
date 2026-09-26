package com.gemwallet.android.features.transactions.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.connection.cases.ObserveRefreshInterval
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.components.filters.TransactionFilterUIModel
import com.gemwallet.android.ui.components.filters.transactionFilterOptions
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.GemTransactionsServiceInterface
import uniffi.gemstone.activityFilters
import uniffi.gemstone.chainsFilterSummary
import uniffi.gemstone.loadError
import uniffi.gemstone.transactionsEmptyState
import uniffi.gemstone.transactionsFilterSummary
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class TransactionsViewModel @Inject constructor(
    getSession: GetSession,
    getTransactions: GetTransactions,
    private val service: GemTransactionsServiceInterface,
    private val observeRefreshInterval: ObserveRefreshInterval,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val refreshIntervalMillis: StateFlow<Long> = observeRefreshInterval.refreshIntervalMillis(GemRefreshKind.WALLET)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing

    val chainsFilter = MutableStateFlow<List<Chain>>(emptyList())

    val typeFilter = MutableStateFlow<List<GemTransactionFilter>>(emptyList())

    val typeFilterOptions: List<TransactionFilterUIModel> = transactionFilterOptions(context)

    val emptyStateKind: StateFlow<GemEmptyStateKind> = combine(chainsFilter, typeFilter) { chains, types ->
        transactionsEmptyState(chains.map { it.string }, types)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, GemEmptyStateKind.ACTIVITY)

    val filterSummary: StateFlow<TransactionsFilterSummaryUIModel> = combine(chainsFilter, typeFilter) { chains, types ->
        TransactionsFilterSummaryUIModel(
            chains = chainsFilterSummary(chains.map { it.string }).text(context),
            types = transactionsFilterSummary(types).text(context),
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, TransactionsFilterSummaryUIModel("", ""))

    val typeFilterRows: StateFlow<List<TransactionFilterUIModel>> = typeFilter
        .map { selected -> typeFilterOptions.filter { it.filter in selected } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val session = getSession()
        .stateIn(viewModelScope, started = SharingStarted.Eagerly, null)

    val walletId: StateFlow<WalletId?> = session
        .map { it?.wallet?.id }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableChains: StateFlow<List<Chain>> = session
        .map { session -> session?.wallet?.let { service.filterChains(it.toGem()).map { chain -> chain.requireChain() } } ?: emptyList() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private var syncedWalletId: WalletId? = null
    private var refreshJob: Job? = null

    val transactions = combine(
        chainsFilter,
        typeFilter,
    ) { chains, types ->
        activityFilters(chains.map { it.string }, types).toPrimitives()
    }
        .flatMapLatest { filter -> getTransactions.getTransactions(filter) }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.Eagerly,
            initialValue = getTransactions.stored(activityFilters(emptyList(), emptyList()).toPrimitives()).takeIf { it.isNotEmpty() },
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
                .drop(1)
                .collect {
                    clearChainsFilter()
                    clearTypeFilter()
                }
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
        chainsFilter.update { chains }
    }

    fun setTypesFilter(types: List<GemTransactionFilter>) {
        typeFilter.update { types }
    }

    fun clearChainsFilter() {
        chainsFilter.update {
            emptyList()
        }
    }

    fun clearTypeFilter() {
        typeFilter.update {
            emptyList()
        }
    }
}

data class TransactionsFilterSummaryUIModel(val chains: String, val types: String)
