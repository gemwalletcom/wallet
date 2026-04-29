package com.gemwallet.android.features.activities.viewmodels

import android.text.format.DateUtils
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.transactions.coordinators.GetTransactions
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ui.models.TransactionTypeFilter
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.onEach
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

    private val _state = MutableStateFlow(true)
    val state: StateFlow<Boolean> = _state

    val chainsFilter = MutableStateFlow<List<Chain>>(emptyList())

    val typeFilter = MutableStateFlow<List<TransactionTypeFilter>>(emptyList())

    val session = sessionRepository.session()
        .stateIn(viewModelScope, started = SharingStarted.Eagerly, null)

    val transactions = combine(
        chainsFilter,
        typeFilter
    ) { chains, types ->
        Pair(chains, types.fold(emptyList<TransactionType>(), { acc, filter -> acc + filter.types }))
    }
    .flatMapLatest { (chains, types) ->
        getTransactions.getTransactions(filterByChains = chains, filterByType = types)
    }
    .onEach {
        _state.update { false }
    }
    .distinctUntilChanged()
    .stateIn(viewModelScope, started = SharingStarted.Eagerly, emptyList())

    init {
        refresh()
    }

    fun refresh() = viewModelScope.launch(Dispatchers.IO) {
        _state.update { true }
        syncTransactions.syncTransactions(session.value?.wallet ?: return@launch)
        viewModelScope.launch(Dispatchers.IO) {
            delay(500)
            _state.update { false }
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
