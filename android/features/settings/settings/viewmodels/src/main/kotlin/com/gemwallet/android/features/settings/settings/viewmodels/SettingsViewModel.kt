package com.gemwallet.android.features.settings.settings.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.application.wallet_connect.cases.IsWalletConnectEnabled
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.NotificationsAvailable
import com.gemwallet.android.ui.localization.text
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemListSection
import uniffi.gemstone.GemPushResult
import uniffi.gemstone.GemSettingsServiceInterface
import uniffi.gemstone.notificationsSections
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val getWallets: GetWallets,
    private val switchPushEnabled: SwitchPushEnabled,
    private val getPushEnabled: GetPushEnabled,
    val notificationsAvailable: NotificationsAvailable,
    private val settingsService: GemSettingsServiceInterface,
    private val isWalletConnectEnabled: IsWalletConnectEnabled,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val wallets = getWallets()
    private val developerEnabled = MutableStateFlow(userConfig.developEnabled())

    val sections = combine(wallets, developerEnabled) { wallets, _ ->
        settingsService.sections(
            wallets = wallets.map { it.toGem() },
            notificationsAvailable = notificationsAvailable,
            walletConnectAvailable = isWalletConnectEnabled.isWalletConnectEnabled(),
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val pushEnabled = getPushEnabled.getPushEnabled()
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    val notificationsSections: StateFlow<List<GemListSection>> = pushEnabled
        .map(::notificationsSections)
        .stateIn(viewModelScope, SharingStarted.Eagerly, notificationsSections(pushEnabled.value))

    fun refreshDeveloperMode() {
        developerEnabled.value = userConfig.developEnabled()
    }

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    fun enableNotifications() = switchNotifications(true)

    fun disableNotifications() = switchNotifications(false)

    fun clearError() = errorState.update { null }

    private fun switchNotifications(enabled: Boolean) {
        viewModelScope.launch(ioDispatcher) {
            val state = switchPushEnabled.switchPushEnabled(enabled)
            errorState.value = (state.result as? GemPushResult.NotRegistered)?.error?.text(context)
        }
    }
}
