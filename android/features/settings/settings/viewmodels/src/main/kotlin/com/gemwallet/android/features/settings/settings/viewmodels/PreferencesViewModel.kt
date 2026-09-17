package com.gemwallet.android.features.settings.settings.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.domains.perpetual.formatLeverage
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.settings.settings.viewmodels.models.PerpetualOptions
import com.gemwallet.android.features.settings.settings.viewmodels.models.PerpetualSetting
import com.gemwallet.android.features.settings.settings.viewmodels.models.PerpetualValues
import com.gemwallet.android.features.settings.settings.viewmodels.models.PickerOption
import com.gemwallet.android.features.settings.settings.viewmodels.models.uiModel
import com.gemwallet.android.math.toUnsignedInts
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Appearance
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.GemSettingsServiceInterface
import uniffi.gemstone.PerpetualProvider

@HiltViewModel
class PreferencesViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val settingsService: GemSettingsServiceInterface,
    getCurrentCurrency: GetCurrentCurrency,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val isPerpetualEnabled = userConfig.isPerpetualEnabled()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private val currency = getCurrentCurrency.getCurrency()

    private val state = combine(currency, isPerpetualEnabled) { currency, perpetualEnabled ->
        settingsService.preferences(currency.toGem(), perpetualEnabled)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, settingsService.preferences(currency.value.toGem(), false))

    private val appearance = userConfig.appearance()
        .stateIn(viewModelScope, SharingStarted.Eagerly, Appearance.System)

    fun setAppearance(appearance: Appearance) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setAppearance(appearance)
    }

    private val perpetualLeverage = userConfig.perpetualLeverage()

    fun setPerpetualEnabled(enabled: Boolean) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualEnabled(enabled)
    }

    private fun setPerpetualLeverage(value: Int) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualLeverage(value)
    }

    private val perpetualTakeProfit = userConfig.perpetualTakeProfit()

    private fun setPerpetualTakeProfit(value: Int) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualTakeProfit(value)
    }

    private val perpetualStopLoss = userConfig.perpetualStopLoss()

    private fun setPerpetualStopLoss(value: Int) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualStopLoss(value)
    }

    private val perpetualOptions = GemPerpetual(PerpetualProvider.HYPERCORE).use { perpetual ->
        PerpetualOptions(
            leverage = perpetual.leverageOptions(null).toUnsignedInts().map { PickerOption(it, it.formatLeverage()) },
            takeProfit = perpetual.takeProfitOptions().toUnsignedInts().map { PickerOption(it, autocloseLabel(perpetual.autoclosePercent(it.toUByte()))) },
            stopLoss = perpetual.stopLossOptions().toUnsignedInts().map { PickerOption(it, autocloseLabel(perpetual.autoclosePercent(it.toUByte()))) },
        )
    }

    private val perpetualValues = combine(isPerpetualEnabled, perpetualLeverage, perpetualTakeProfit, perpetualStopLoss) { enabled, leverage, takeProfit, stopLoss ->
        PerpetualValues(enabled, leverage, takeProfit, stopLoss)
    }

    val rows = combine(state, appearance, perpetualValues) { state, appearance, perpetual ->
        state.sections.map { section -> section.rows.map { it.uiModel(context, state, appearance, perpetual, perpetualOptions) } }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun setPerpetualOption(setting: PerpetualSetting, value: Int) = when (setting) {
        PerpetualSetting.Leverage -> setPerpetualLeverage(value)
        PerpetualSetting.TakeProfit -> setPerpetualTakeProfit(value)
        PerpetualSetting.StopLoss -> setPerpetualStopLoss(value)
    }

    private fun autocloseLabel(percent: UByte?): String = percent?.let { "$it%" } ?: context.getString(R.string.common_none)
}
