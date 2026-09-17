package com.gemwallet.android.features.settings.currency.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.SetCurrentCurrency
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.settings.currency.viewmodels.models.sections
import com.wallet.core.primitives.Currency
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import java.util.Locale
import javax.inject.Inject
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemCurrencyServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class CurrenciesViewModel @Inject constructor(
    private val service: GemCurrencyServiceInterface,
    getCurrentCurrency: GetCurrentCurrency,
    private val setCurrentCurrency: SetCurrentCurrency,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {
    private val localeCurrency: Currency? = runCatching { java.util.Currency.getInstance(Locale.getDefault()).currencyCode }
        .getOrNull()
        ?.let { Currency.entries.firstOrNull { currency -> currency.string == it } }

    private val currency = getCurrentCurrency.getCurrency()

    val sections = currency.mapLatest { service.currencies(localeCurrency?.toGem()).sections(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun setCurrency(currency: Currency) {
        if (this.currency.value == currency) {
            return
        }

        setCurrentCurrency.setCurrentCurrency(currency)
    }
}
