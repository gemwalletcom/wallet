package com.gemwallet.android.features.settings.settings.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Appearance
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject
import uniffi.gemstone.GemSettingsServiceInterface

@HiltViewModel
class PreferencesViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val settingsService: GemSettingsServiceInterface,
    getCurrentCurrency: GetCurrentCurrency,
) : ViewModel() {

    val isPerpetualEnabled = userConfig.isPerpetualEnabled()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private val currency = getCurrentCurrency.getCurrency()

    val state = combine(currency, isPerpetualEnabled) { currency, perpetualEnabled ->
        settingsService.preferences(currency.toGem(), perpetualEnabled)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, settingsService.preferences(currency.value.toGem(), false))

    val appearance = userConfig.appearance()
        .stateIn(viewModelScope, SharingStarted.Eagerly, Appearance.System)

    fun setAppearance(appearance: Appearance) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setAppearance(appearance)
    }

    val perpetualLeverage = userConfig.perpetualLeverage()

    fun setPerpetualEnabled(enabled: Boolean) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualEnabled(enabled)
    }

    fun setPerpetualLeverage(value: Int) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualLeverage(value)
    }

    val perpetualTakeProfit = userConfig.perpetualTakeProfit()

    fun setPerpetualTakeProfit(value: Int) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualTakeProfit(value)
    }

    val perpetualStopLoss = userConfig.perpetualStopLoss()

    fun setPerpetualStopLoss(value: Int) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualStopLoss(value)
    }
}
