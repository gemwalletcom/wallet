package com.gemwallet.android.features.activities.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import com.gemwallet.android.domains.connection.refreshInterval
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.components.filters.TransactionFilterUIModel
import com.gemwallet.android.ui.components.filters.transactionFilterOptions
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.CoroutineDispatcher
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
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.GemTransactionsEmptyState
import uniffi.gemstone.GemTransactionsServiceInterface
import uniffi.gemstone.transactionsEmptyState

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class TransactionsViewModel @Inject constructor(
    getSession: GetSession,
    getTransactions: GetTransactions,
    private val service: GemTransactionsServiceInterface,
    private val connectionStatusObserver: ConnectionStatusObserver,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val refreshIntervalMillis: StateFlow<Long> = connectionStatusObserver.status
        .map { it.refreshInterval(GemRefreshKind.WALLET).toMillis() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)


    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing

    val chainsFilter = MutableStateFlow<List<Chain>>(emptyList())

    val typeFilter = MutableStateFlow<List<GemTransactionFilter>>(emptyList())

    val typeFilterOptions: List<TransactionFilterUIModel> = transactionFilterOptions(context)

    val showsNoResults: StateFlow<Boolean> = combine(chainsFilter, typeFilter) { chains, types ->
        transactionsEmptyState(chains.map { it.string }, types) == GemTransactionsEmptyState.NO_RESULTS
    }.stateIn(viewModelScope, SharingStarted.Eagerly, false)

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

    val transactions = combine(
        chainsFilter,
        typeFilter,
    ) { chains, types ->
        TransactionsRequestFilter.activity(chains, types)
    }
    .flatMapLatest { filters -> getTransactions.getTransactions(filters) }
    .stateIn(
        scope = viewModelScope,
        started = SharingStarted.Eagerly,
        initialValue = null,
    )

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
        return viewModelScope.launch(ioDispatcher) {
            val synced = sync()
            if (!synced && syncedWalletId == current) {
                syncedWalletId = null
            }
        }
    }

    private suspend fun sync(): Boolean = runCatchingCancellable { service.sync(null) }
        .onFailure { Log.e(TAG, "transactions sync failed", it) }
        .isSuccess

    fun refresh() = viewModelScope.launch(ioDispatcher) {
        _isRefreshing.update { true }
        try {
            sync()
        } finally {
            _isRefreshing.update { false }
        }
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

    private companion object {
        const val TAG = "TransactionsViewModel"
    }
}
