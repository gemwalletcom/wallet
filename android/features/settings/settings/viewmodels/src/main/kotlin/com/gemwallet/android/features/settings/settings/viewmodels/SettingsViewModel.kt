package com.gemwallet.android.features.settings.settings.viewmodels

import com.gemwallet.android.ext.toGem
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.model.NotificationsAvailable
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.GemSettingsServiceInterface
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val getWallets: GetWallets,
    private val switchPushEnabled: SwitchPushEnabled,
    private val getPushEnabled: GetPushEnabled,
    val notificationsAvailable: NotificationsAvailable,
    private val settingsService: GemSettingsServiceInterface,
) : ViewModel() {

    private val wallets = getWallets()
    private val developerEnabled = MutableStateFlow(userConfig.developEnabled())
    private val walletConnectAvailable = MutableStateFlow(true)

    val sections = combine(wallets, developerEnabled, walletConnectAvailable) { wallets, _, walletConnect ->
        settingsService.sections(
            wallets = wallets.map { it.toGem() },
            notificationsAvailable = notificationsAvailable,
            walletConnectAvailable = walletConnect,
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun setWalletConnectAvailable(available: Boolean) {
        walletConnectAvailable.value = available
    }

    val walletsCount = wallets.map { it.size }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    val pushEnabled = getPushEnabled.getPushEnabled()
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    fun developEnable() {
        userConfig.developEnabled(!userConfig.developEnabled())
        developerEnabled.value = userConfig.developEnabled()
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
