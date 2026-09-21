package com.gemwallet.android.features.settings.currency.viewmodels

import android.content.Context
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.SetCurrentCurrency
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.settings.currency.viewmodels.models.uiModel
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Currency
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemCurrencyServiceInterface
import java.util.Locale
import javax.inject.Inject

@HiltViewModel
class CurrenciesViewModel @Inject constructor(
    private val service: GemCurrencyServiceInterface,
    getCurrentCurrency: GetCurrentCurrency,
    private val setCurrentCurrency: SetCurrentCurrency,
    @param:ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {
    private val localeCurrency: Currency? = runCatching { java.util.Currency.getInstance(Locale.getDefault()).currencyCode }
        .getOrNull()
        ?.let { Currency.entries.firstOrNull { currency -> currency.string == it } }

    val query = TextFieldState()

    val sections = combine(getCurrentCurrency.getCurrency(), snapshotFlow { query.text.toString() }) { currency, query ->
        val localizedNames = Currency.entries.associate { it.string to android.icu.util.Currency.getInstance(it.string).displayName }
        service.sections(currency.toGem(), localeCurrency?.toGem(), query, localizedNames).map { it.uiModel(context) }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    fun setCurrency(currency: Currency, onSelected: () -> Unit) = viewModelScope.launch {
        runCatchingCancellable { setCurrentCurrency.setCurrentCurrency(currency) }
            .onSuccess { onSelected() }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun clearError() = errorState.update { null }
}
