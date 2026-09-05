package com.gemwallet.android.features.activities.viewmodels

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.serializer.toJson
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ui.models.TransactionTypeFilter
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAssetConfigServiceInterface
import uniffi.gemstone.GemTransactionsServiceInterface
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.requireChain
import android.util.Log
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class TransactionsViewModel @Inject constructor(
    getSession: GetSession,
    getTransactions: GetTransactions,
    private val service: GemTransactionsServiceInterface,
    private val assetConfig: GemAssetConfigServiceInterface,
) : ViewModel() {

    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing

    val chainsFilter = MutableStateFlow<List<Chain>>(emptyList())

    val typeFilter = MutableStateFlow<List<TransactionTypeFilter>>(emptyList())

    val session = getSession()
        .stateIn(viewModelScope, started = SharingStarted.Eagerly, null)

    val walletId: StateFlow<WalletId?> = session
        .map { it?.wallet?.id }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableChains: StateFlow<List<Chain>> = session
        .map { session -> session?.wallet?.let { service.filterChains(it.toJson()).map { chain -> chain.requireChain() } } ?: emptyList() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private var syncedWalletId: WalletId? = null

    val transactions = combine(
        chainsFilter,
        typeFilter,
    ) { chains, types ->
        buildList {
            addAll(TransactionsRequestFilter.activityDefaults(assetConfig))
            if (chains.isNotEmpty()) add(TransactionsRequestFilter.Chains(chains))
            val allowedTypes = types.flatMap { it.types }
            if (allowedTypes.isNotEmpty()) add(TransactionsRequestFilter.Types(allowedTypes))
        }
    }
    .flatMapLatest { filters -> getTransactions.getTransactions(filters) }
    .stateIn(
        scope = viewModelScope,
        started = SharingStarted.Eagerly,
        initialValue = getTransactions.transactions().value,
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
        return viewModelScope.launch(Dispatchers.IO) {
            val synced = sync()
            if (!synced && syncedWalletId == current) {
                syncedWalletId = null
            }
        }
    }

    private suspend fun sync(): Boolean = runCatchingCancellable { service.sync(null) }
        .onFailure { Log.e(TAG, "transactions sync failed", it) }
        .isSuccess

    fun refresh() = viewModelScope.launch(Dispatchers.IO) {
        _isRefreshing.update { true }
        try {
            sync()
        } finally {
            _isRefreshing.update { false }
        }
    }

    fun applyChainsFilter(chains: List<Chain>) {
        chainsFilter.update { chains }
    }

    fun applyTypesFilter(types: List<TransactionTypeFilter>) {
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
