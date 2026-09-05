package com.gemwallet.android.features.settings.settings.viewmodels

import com.gemwallet.android.ext.toGem
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.model.NotificationsAvailable
import com.wallet.core.primitives.Appearance
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.GemWalletSessionServiceInterface
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val getWallets: GetWallets,
    private val getSession: GetSession,
    private val getCurrentCurrency: GetCurrentCurrency,
    private val switchPushEnabled: SwitchPushEnabled,
    private val getPushEnabled: GetPushEnabled,
    val notificationsAvailable: NotificationsAvailable,
    private val walletSessionService: GemWalletSessionServiceInterface,
) : ViewModel() {

    private val session = getSession()
    private val wallets = getWallets()
    private val state = MutableStateFlow(SettingsViewModelState(currency = getCurrentCurrency.getCurrency().value))
    val uiState = state.asStateFlow()

    val isRewardsAvailable = wallets
        .map { wallets -> walletSessionService.showsRewards(wallets.map { it.toGem() }) }
        .stateIn(
            viewModelScope,
            SharingStarted.Eagerly,
            true,
        )

    val walletsCount = wallets.map { it.size }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    val pushEnabled = getPushEnabled.getPushEnabled()
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    val isPerpetualEnabled = userConfig.isPerpetualEnabled()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val appearance = userConfig.appearance()
        .stateIn(viewModelScope, SharingStarted.Eagerly, Appearance.System)

    fun setAppearance(appearance: Appearance) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setAppearance(appearance)
    }

    val perpetualLeverage = userConfig.perpetualLeverage()
        .stateIn(viewModelScope, SharingStarted.Eagerly, PerpetualConfig.defaultLeverage)

    fun setPerpetualEnabled(enabled: Boolean) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualEnabled(enabled)
    }

    fun setPerpetualLeverage(value: Int) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualLeverage(value)
    }

    val perpetualTakeProfit = userConfig.perpetualTakeProfit()
        .stateIn(viewModelScope, SharingStarted.Eagerly, PerpetualConfig.defaultTakeProfit)

    fun setPerpetualTakeProfit(value: Int) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualTakeProfit(value)
    }

    val perpetualStopLoss = userConfig.perpetualStopLoss()
        .stateIn(viewModelScope, SharingStarted.Eagerly, PerpetualConfig.defaultStopLoss)

    fun setPerpetualStopLoss(value: Int) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setPerpetualStopLoss(value)
    }

    init {
        viewModelScope.launch {
            session.collectLatest {
                refresh()
            }
        }
        refresh()
    }

    private fun refresh() = viewModelScope.launch(Dispatchers.IO) {
        state.update {
            it.copy(
                currency = getCurrentCurrency.getCurrency().value,
                developEnabled = userConfig.developEnabled(),
            )
        }
    }

    fun developEnable() {
        userConfig.developEnabled(!userConfig.developEnabled())
        refresh()
    }

    fun enableNotifications() {
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.stopAskNotifications()
            switchPushEnabled.switchPushEnabled(true)
        }
    }

    fun disableNotifications() {
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.stopAskNotifications()
            switchPushEnabled.switchPushEnabled(false)
        }
    }

}

data class SettingsViewModelState(
    val currency: Currency,
    val developEnabled: Boolean = false,
)
