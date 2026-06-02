package com.gemwallet.android.features.settings.settings.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.cases.device.GetPushEnabled
import com.gemwallet.android.cases.device.SwitchPushEnabled
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.model.NotificationsAvailable
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val walletsRepository: WalletsRepository,
    private val sessionRepository: SessionRepository,
    private val switchPushEnabled: SwitchPushEnabled,
    private val getPushEnabled: GetPushEnabled,
    private val notificationsAvailable: NotificationsAvailable,
) : ViewModel() {

    private val session = sessionRepository.session()
    private val wallets = walletsRepository.getAll()
    private val state = MutableStateFlow(SettingsViewModelState())
    val uiState = state.map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, SettingsUIState.General())

    val isRewardsAvailable = wallets
        .map { items ->
            items.isEmpty() || items.any { it.type == WalletType.Multicoin }
        }
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
                currency = session.value?.currency ?: Currency.USD,
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
            switchPushEnabled.switchPushEnabled(
                true,
                wallets.firstOrNull() ?: emptyList()
            )
        }
    }

    fun disableNotifications() {
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.stopAskNotifications()
            switchPushEnabled.switchPushEnabled(
                false,
                wallets.firstOrNull() ?: emptyList()
            )
        }
    }

    fun isNotificationsAvailable(): Boolean {
        return notificationsAvailable
    }
}

data class SettingsViewModelState(
    val currency: Currency = Currency.USD,
    val developEnabled: Boolean = false,
) {
    fun toUIState(): SettingsUIState.General {
        return SettingsUIState.General(
            currency = currency,
            developEnabled = developEnabled,
        )
    }
}

sealed interface SettingsUIState {

    data class General(
        val currency: Currency = Currency.USD,
        val developEnabled: Boolean = false,
    ) : SettingsUIState
}
