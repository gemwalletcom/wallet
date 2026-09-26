package com.gemwallet.android.features.settings.viewmodels

import android.content.Context
import android.content.res.Configuration
import android.os.Build
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.preferences.cases.ObservablePreferences
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.settings.viewmodels.models.PerpetualSetting
import com.gemwallet.android.features.settings.viewmodels.models.value
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.stringRes
import com.wallet.core.primitives.Appearance
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemPreferencesInput
import uniffi.gemstone.GemSettingsServiceInterface
import java.util.Locale
import javax.inject.Inject

@HiltViewModel
class PreferencesViewModel @Inject constructor(
    private val preferences: ObservablePreferences,
    private val settingsService: GemSettingsServiceInterface,
    getCurrentCurrency: GetCurrentCurrency,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val isPerpetualEnabled = preferences.isPerpetualEnabled()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private val currency = getCurrentCurrency.getCurrency()

    private val language = MutableStateFlow(languageText { context.resources.configuration })

    val appearance = preferences.appearance()
        .stateIn(viewModelScope, SharingStarted.Eagerly, Appearance.System)

    val perpetualDefaults = MutableStateFlow(settingsService.perpetualDefaults())

    val perpetualOptions = settingsService.perpetualPickers()

    val sections = combine(currency, isPerpetualEnabled, appearance, perpetualDefaults, language) { currency, perpetualsEnabled, appearance, defaults, language ->
        settingsService.preferencesSections(
            GemPreferencesInput(
                currency = currency.toGem(),
                language = language,
                appearance = context.getString(appearance.stringRes()),
                perpetualsEnabled = perpetualsEnabled,
                perpetualDefaults = defaults,
            ),
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun setLanguage(configuration: Configuration) {
        language.value = languageText { configuration }
    }

    fun setAppearance(appearance: Appearance) = viewModelScope.launch(ioDispatcher) {
        preferences.setAppearance(appearance)
    }

    fun setPerpetualEnabled(enabled: Boolean) = viewModelScope.launch(ioDispatcher) {
        preferences.setPerpetualEnabled(enabled)
    }

    fun setPerpetualOption(setting: PerpetualSetting, value: Int) = viewModelScope.launch(ioDispatcher) {
        val defaults = perpetualDefaults.value
        val updated = when (setting) {
            PerpetualSetting.Leverage -> defaults.copy(leverage = value.toUByte())
            PerpetualSetting.TakeProfit -> defaults.copy(takeProfitPercent = value.toUByte())
            PerpetualSetting.StopLoss -> defaults.copy(stopLossPercent = value.toUByte())
        }
        runCatchingCancellable { settingsService.setPerpetualDefaults(updated) }
            .onSuccess { perpetualDefaults.value = updated }
            .onFailure { Log.e(TAG, "saving the perpetual defaults failed", it) }
    }

    private fun languageText(configuration: () -> Configuration): String? = when {
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU -> configuration().locales.get(0).displayLanguage.replaceFirstChar {
            if (it.isLowerCase()) it.titlecase(Locale.ROOT) else it.toString()
        }

        else -> null
    }
}

private const val TAG = "Preferences"
