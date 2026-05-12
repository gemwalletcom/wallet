package com.gemwallet.android.features.activities.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.transactions.coordinators.GetTransactions
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ui.models.TransactionTypeFilter
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class TransactionsViewModel @Inject constructor(
    sessionRepository: SessionRepository,
    getTransactions: GetTransactions,
    private val syncTransactions: SyncTransactions,
) : ViewModel() {

    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing

    val chainsFilter = MutableStateFlow<List<Chain>>(emptyList())

    val typeFilter = MutableStateFlow<List<TransactionTypeFilter>>(emptyList())

    val session = sessionRepository.session()
        .stateIn(viewModelScope, started = SharingStarted.Eagerly, null)

    val walletId: StateFlow<WalletId?> = session
        .map { it?.wallet?.id }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private var lastSyncedWalletId: WalletId? = null

    val transactions = combine(
        getTransactions.transactions(),
        chainsFilter,
        typeFilter,
    ) { items, chains, types ->
        val allowedTypes = types.flatMap { it.types }.toSet()
        items.filter {
            (chains.isEmpty() || it.asset.id.chain in chains) &&
                (allowedTypes.isEmpty() || it.type in allowedTypes)
        }
    }
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

    fun syncIfNeeded() {
        val current = walletId.value ?: return
        if (current == lastSyncedWalletId) return
        lastSyncedWalletId = current
        viewModelScope.launch(Dispatchers.IO) {
            val wallet = session.firstOrNull()?.wallet ?: return@launch
            syncTransactions.syncTransactions(wallet)
        }
    }

    fun refresh() = viewModelScope.launch(Dispatchers.IO) {
        val wallet = session.firstOrNull()?.wallet ?: return@launch
        _isRefreshing.update { true }
        try {
            syncTransactions.syncTransactions(wallet)
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
}
