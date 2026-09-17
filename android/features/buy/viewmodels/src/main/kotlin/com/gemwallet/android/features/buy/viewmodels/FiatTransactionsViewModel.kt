package com.gemwallet.android.features.buy.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.cases.ObserveFiatTransactions
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.features.buy.viewmodels.models.FiatTransactionRowUIModel
import com.gemwallet.android.features.buy.viewmodels.models.uiModel
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemFiatQuoteServiceInterface

@HiltViewModel
class FiatTransactionsViewModel @Inject constructor(
    observeFiatTransactions: ObserveFiatTransactions,
    private val service: GemFiatQuoteServiceInterface,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing
    val transactions: StateFlow<List<FiatTransactionRowUIModel>> = observeFiatTransactions()
        .map { items -> items.map { it.uiModel(context) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    init {
        refresh()
    }

    fun refresh() = viewModelScope.launch(Dispatchers.IO) {
        _isRefreshing.value = true
        try {
            sync()
        } finally {
            _isRefreshing.value = false
        }
    }

    private suspend fun sync() {
        runCatchingCancellable { service.syncTransactions() }
            .onFailure { Log.e(TAG, "fiat transactions sync failed", it) }
    }

    private companion object {
        const val TAG = "FiatTransactions"
    }
}
