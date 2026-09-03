package com.gemwallet.android.features.buy.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.cases.ObserveFiatTransactions
import com.wallet.core.primitives.FiatTransactionAssetData
import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import uniffi.gemstone.GemFiatQuoteServiceInterface
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class FiatTransactionsViewModel @Inject constructor(
    observeFiatTransactions: ObserveFiatTransactions,
    private val service: GemFiatQuoteServiceInterface,
) : ViewModel() {

    val isRefreshing = MutableStateFlow(false)
    val transactions: StateFlow<List<FiatTransactionAssetData>> = observeFiatTransactions()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    init {
        viewModelScope.launch(Dispatchers.IO) { sync() }
    }

    fun refresh() = viewModelScope.launch(Dispatchers.IO) {
        isRefreshing.value = true
        try {
            sync()
        } finally {
            isRefreshing.value = false
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
