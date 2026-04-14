package com.gemwallet.android.features.settings.settings.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.cases.device.GetPushEnabled
import com.gemwallet.android.cases.device.SwitchPushEnabled
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.model.NotificationsAvailable
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val walletsRepository: WalletsRepository,
    private val sessionRepository: SessionRepository,
    private val switchPushEnabled: SwitchPushEnabled,
    private val getPushEnabled: GetPushEnabled,
    private val notificationsAvailable: NotificationsAvailable,
) : ViewModel() {

    private val state = MutableStateFlow(SettingsViewModelState())
    val uiState = state.map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, SettingsUIState.General())

    val isRewardsAvailable = walletsRepository.getAll().mapLatest { items -> items.any { it.type == WalletType.Multicoin } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    val walletsCount = walletsRepository.getAll().mapLatest { it.size }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    val pushEnabled = getPushEnabled.getPushEnabled()
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    init {
        viewModelScope.launch {
            sessionRepository.session().collectLatest {
                refresh()
            }
        }
        refresh()
    }

    private fun refresh() = viewModelScope.launch(Dispatchers.IO) {
        state.update {
            it.copy(
                currency = sessionRepository.session().firstOrNull()?.currency ?: Currency.USD,
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
                walletsRepository.getAll().firstOrNull() ?: emptyList()
            )
        }
    }

    fun disableNotifications() {
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.stopAskNotifications()
            switchPushEnabled.switchPushEnabled(
                false,
                walletsRepository.getAll().firstOrNull() ?: emptyList()
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
