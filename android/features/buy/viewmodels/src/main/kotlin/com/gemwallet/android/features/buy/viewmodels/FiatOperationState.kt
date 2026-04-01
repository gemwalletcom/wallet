package com.gemwallet.android.features.buy.viewmodels

import com.gemwallet.android.features.buy.viewmodels.models.FiatSceneState
import com.wallet.core.primitives.FiatQuote
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

class FiatOperationState(
    val defaultAmount: String,
    val minFiatAmount: Double,
) {
    private val _amount = MutableStateFlow(defaultAmount)
    val amount: StateFlow<String> get() = _amount

    private val _state = MutableStateFlow<FiatSceneState?>(null)
    val state: StateFlow<FiatSceneState?> get() = _state

    private val _quotes = MutableStateFlow<List<FiatQuote>>(emptyList())
    val quotes: StateFlow<List<FiatQuote>> get() = _quotes

    private val _selectedQuote = MutableStateFlow<FiatQuote?>(null)
    val selectedQuote: StateFlow<FiatQuote?> get() = _selectedQuote

    fun updateAmount(value: String) {
        _amount.value = value
    }

    fun updateState(value: FiatSceneState?) {
        _state.value = value
    }

    fun updateQuotes(value: List<FiatQuote>) {
        _quotes.value = value
        _selectedQuote.value = value.firstOrNull()
    }

    fun clearQuotes() {
        _quotes.value = emptyList()
        _selectedQuote.value = null
    }

    fun selectProvider(providerName: String) {
        _selectedQuote.value = _quotes.value.firstOrNull { it.provider.name == providerName }
    }
}
