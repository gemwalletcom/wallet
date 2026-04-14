package com.gemwallet.android.features.buy.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.coordinators.ObserveFiatTransactions
import com.gemwallet.android.application.fiat.coordinators.SyncFiatTransactions
import com.wallet.core.primitives.FiatTransactionAssetData
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
    private val syncFiatTransactions: SyncFiatTransactions,
) : ViewModel() {

    val isRefreshing = MutableStateFlow(false)
    val transactions: StateFlow<List<FiatTransactionAssetData>> = observeFiatTransactions()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    init {
        viewModelScope.launch(Dispatchers.IO) {
            syncFiatTransactions()
        }
    }

    fun refresh() = viewModelScope.launch(Dispatchers.IO) {
        isRefreshing.value = true
        try {
            syncFiatTransactions()
        } finally {
            isRefreshing.value = false
        }
    }
}
