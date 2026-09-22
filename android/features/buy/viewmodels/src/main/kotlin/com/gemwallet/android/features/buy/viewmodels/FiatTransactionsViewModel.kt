package com.gemwallet.android.features.buy.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.fiat.cases.ObserveFiatTransactions
import com.gemwallet.android.features.buy.viewmodels.models.FiatTransactionRowUIModel
import com.gemwallet.android.features.buy.viewmodels.models.uiModel
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemFiatQuoteServiceInterface
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.loadError
import javax.inject.Inject

@HiltViewModel
class FiatTransactionsViewModel @Inject constructor(
    observeFiatTransactions: ObserveFiatTransactions,
    private val service: GemFiatQuoteServiceInterface,
    @param:ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing

    private val loadState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)
    val transactions: StateFlow<List<FiatTransactionRowUIModel>> = observeFiatTransactions()
        .map { items -> items.map { it.uiModel(context) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val errorRow: StateFlow<GemListRow?> = combine(loadState, transactions) { state, items ->
        loadError(state, items.isNotEmpty())?.let { GemListRow.Error(it) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    init {
        refresh()
    }

    fun refresh() = viewModelScope.launch(ioDispatcher) {
        _isRefreshing.value = true
        try {
            sync()
        } finally {
            _isRefreshing.value = false
        }
    }

    private suspend fun sync() {
        loadState.update { service.refreshTransactions(transactions.value.isNotEmpty()) }
    }
}
