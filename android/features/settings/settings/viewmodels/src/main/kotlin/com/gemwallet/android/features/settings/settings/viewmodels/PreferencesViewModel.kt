package com.gemwallet.android.features.settings.settings.viewmodels

import android.content.Context
import android.content.res.Configuration
import android.os.Build
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import com.gemwallet.android.domains.perpetual.formatLeverage
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.settings.settings.viewmodels.localization.stringRes
import com.gemwallet.android.features.settings.settings.viewmodels.models.PerpetualOptions
import com.gemwallet.android.features.settings.settings.viewmodels.models.PerpetualSetting
import com.gemwallet.android.features.settings.settings.viewmodels.models.PickerOption
import com.gemwallet.android.features.settings.settings.viewmodels.models.value
import com.gemwallet.android.math.toUnsignedInts
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Appearance
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import java.util.Locale
import javax.inject.Inject
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.GemPreferencesInput
import uniffi.gemstone.GemSettingsServiceInterface
import uniffi.gemstone.PerpetualProvider

@HiltViewModel
class PreferencesViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val settingsService: GemSettingsServiceInterface,
    getCurrentCurrency: GetCurrentCurrency,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val isPerpetualEnabled = userConfig.isPerpetualEnabled()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private val currency = getCurrentCurrency.getCurrency()

    private val language = MutableStateFlow(languageText { context.resources.configuration })

    val appearance = userConfig.appearance()
        .stateIn(viewModelScope, SharingStarted.Eagerly, Appearance.System)

    val perpetualDefaults = MutableStateFlow(settingsService.perpetualDefaults())

    val perpetualOptions = GemPerpetual(PerpetualProvider.HYPERCORE).use { perpetual ->
        PerpetualOptions(
            leverage = perpetual.leverageOptions(null).toUnsignedInts().map { PickerOption(it, it.formatLeverage()) },
            takeProfit = perpetual.takeProfitOptions().toUnsignedInts().map { PickerOption(it, autocloseLabel(perpetual.autoclosePercent(it.toUByte()))) },
            stopLoss = perpetual.stopLossOptions().toUnsignedInts().map { PickerOption(it, autocloseLabel(perpetual.autoclosePercent(it.toUByte()))) },
        )
    }

    val sections = combine(currency, isPerpetualEnabled, appearance, perpetualDefaults, language) { currency, perpetualsEnabled, appearance, defaults, language ->
        settingsService.preferencesSections(
            GemPreferencesInput(
                currency = currency.toGem(),
                language = language,
                appearance = context.getString(appearance.stringRes()),
                perpetualsEnabled = perpetualsEnabled,
                perpetualLeverage = optionLabel(PerpetualSetting.Leverage, defaults.value(PerpetualSetting.Leverage)),
                perpetualTakeProfit = optionLabel(PerpetualSetting.TakeProfit, defaults.value(PerpetualSetting.TakeProfit)),
                perpetualStopLoss = optionLabel(PerpetualSetting.StopLoss, defaults.value(PerpetualSetting.StopLoss)),
            )
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun setLanguage(configuration: Configuration) {
        language.value = languageText { configuration }
    }

    fun setAppearance(appearance: Appearance) = viewModelScope.launch(ioDispatcher) {
        userConfig.setAppearance(appearance)
    }

    fun setPerpetualEnabled(enabled: Boolean) = viewModelScope.launch(ioDispatcher) {
        userConfig.setPerpetualEnabled(enabled)
    }

    fun setPerpetualOption(setting: PerpetualSetting, value: Int) = viewModelScope.launch(ioDispatcher) {
        val defaults = perpetualDefaults.value
        val updated = when (setting) {
            PerpetualSetting.Leverage -> defaults.copy(leverage = value.toUByte())
            PerpetualSetting.TakeProfit -> defaults.copy(takeProfitPercent = value.toUByte())
            PerpetualSetting.StopLoss -> defaults.copy(stopLossPercent = value.toUByte())
        }
        settingsService.setPerpetualDefaults(updated)
        perpetualDefaults.value = updated
    }

    private fun optionLabel(setting: PerpetualSetting, value: Int): String =
        perpetualOptions.of(setting).firstOrNull { it.value == value }?.label.orEmpty()

    private fun autocloseLabel(percent: UByte?): String = percent?.let { "$it%" } ?: context.getString(R.string.common_none)

    private fun languageText(configuration: () -> Configuration): String? = when {
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU -> configuration().locales.get(0).displayLanguage.replaceFirstChar {
            if (it.isLowerCase()) it.titlecase(Locale.ROOT) else it.toString()
        }
        else -> null
    }
}
