package com.gemwallet.android.features.settings.settings.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.NotificationsAvailable
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemSettingsServiceInterface
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val getWallets: GetWallets,
    private val switchPushEnabled: SwitchPushEnabled,
    private val getPushEnabled: GetPushEnabled,
    val notificationsAvailable: NotificationsAvailable,
    private val settingsService: GemSettingsServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val wallets = getWallets()
    private val developerEnabled = MutableStateFlow(userConfig.developEnabled())
    val isDeveloperEnabled = developerEnabled.asStateFlow()
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

    val notificationsListItem = ListItemModel(title = context.getString(R.string.settings_notifications_title))

    val priceAlertsListItem = ListItemModel(
        title = context.getString(R.string.settings_price_alerts_title),
        image = ListItemImage.Drawable(R.drawable.settings_pricealert),
    )

    val pushEnabled = getPushEnabled.getPushEnabled()
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    fun toggleDeveloperMode() {
        userConfig.developEnabled(!userConfig.developEnabled())
        developerEnabled.value = userConfig.developEnabled()
    }

    fun enableNotifications() {
        viewModelScope.launch(ioDispatcher) {
            switchPushEnabled.switchPushEnabled(true)
        }
    }

    fun disableNotifications() {
        viewModelScope.launch(ioDispatcher) {
            switchPushEnabled.switchPushEnabled(false)
        }
    }
}
