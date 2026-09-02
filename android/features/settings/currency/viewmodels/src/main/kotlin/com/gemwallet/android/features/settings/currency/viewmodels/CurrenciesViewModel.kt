package com.gemwallet.android.features.settings.currency.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.SetCurrentCurrency
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Currency
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemCurrencyServiceInterface
import java.util.Locale
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class CurrenciesViewModel @Inject constructor(
    private val service: GemCurrencyServiceInterface,
    getCurrentCurrency: GetCurrentCurrency,
    private val setCurrentCurrency: SetCurrentCurrency,
) : ViewModel() {
    private val localeCurrency: Currency? = runCatching { java.util.Currency.getInstance(Locale.getDefault()).currencyCode }
        .getOrNull()
        ?.let { Currency.entries.firstOrNull { currency -> currency.string == it } }

    val currency = getCurrentCurrency.getCurrency()
        .stateIn(viewModelScope, SharingStarted.Eagerly, Currency.USD)

    val recommendedCurrencies = currency.mapLatest {
        service.recommendedCurrencies(localeCurrency?.toGem()).map { it.toCurrency() }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val otherCurrencies = currency.mapLatest {
        service.otherCurrencies(localeCurrency?.toGem()).map { it.toCurrency() }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun setCurrency(currency: Currency) {
        if (this.currency.value == currency) {
            return
        }

        setCurrentCurrency.setCurrentCurrency(currency)
    }
}
