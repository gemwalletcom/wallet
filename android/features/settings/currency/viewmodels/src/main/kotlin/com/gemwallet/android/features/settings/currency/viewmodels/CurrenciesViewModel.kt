package com.gemwallet.android.features.settings.currency.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.application.session.coordinators.SetCurrentCurrency
import com.wallet.core.primitives.Currency
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class CurrenciesViewModel @Inject constructor(
    getCurrentCurrency: GetCurrentCurrency,
    private val setCurrentCurrency: SetCurrentCurrency,
) : ViewModel() {
    private val defaultCurrency: List<Currency> = listOf(
        Currency.USD,
        Currency.EUR,
        Currency.GBP,
        Currency.CNY,
        Currency.JPY,
        Currency.INR,
        Currency.RUB,
    )

    val currency = getCurrentCurrency.getCurrency()
        .stateIn(viewModelScope, SharingStarted.Eagerly, Currency.USD)

    val defaultCurrencies = currency.mapLatest {
        if (defaultCurrency.contains(it)) {
            defaultCurrency
        } else {
            listOf(it) + defaultCurrency
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun getCurrencies(): List<Currency> {
        return (Currency.entries.toSet() - defaultCurrency.toSet()).toList()
    }

    fun setCurrency(currency: Currency) {
        if (this.currency.value == currency) {
            return
        }

        setCurrentCurrency.setCurrentCurrency(currency)
    }
}
