package com.gemwallet.android.features.buy.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.fiat.coordinators.ObserveBuyTransactions
import com.gemwallet.android.application.fiat.coordinators.RefreshBuyTransactions
import com.wallet.core.primitives.FiatTransactionAssetData
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class FiatTransactionsViewModel @Inject constructor(
    observeBuyTransactions: ObserveBuyTransactions,
    private val refreshBuyTransactions: RefreshBuyTransactions,
) : ViewModel() {

    val transactions: StateFlow<List<FiatTransactionAssetData>> = observeBuyTransactions()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    init {
        refresh()
    }

    fun refresh() {
        viewModelScope.launch(Dispatchers.IO) {
            refreshBuyTransactions()
        }
    }
}
