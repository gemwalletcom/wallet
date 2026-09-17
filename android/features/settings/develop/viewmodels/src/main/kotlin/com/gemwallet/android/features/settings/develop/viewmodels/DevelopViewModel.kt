package com.gemwallet.android.features.settings.develop.viewmodels

import com.wallet.core.primitives.WalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toPrimitives
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.model.NotificationsAvailable
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.PlatformStore
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject
import uniffi.gemstone.GemDeveloperServiceInterface

@HiltViewModel
class DevelopViewModel @Inject constructor(
    private val service: GemDeveloperServiceInterface,
    private val getSession: GetSession,
    val notificationsAvailable: NotificationsAvailable,
) : ViewModel() {

    private val _deviceId = MutableStateFlow("")
    val deviceId = _deviceId.asStateFlow()
    private val _pushToken = MutableStateFlow("")
    val pushToken = _pushToken.asStateFlow()
    private val _platformStore = MutableStateFlow<PlatformStore?>(null)
    val platformStore = _platformStore.asStateFlow()

    init {
        viewModelScope.launch(Dispatchers.IO) {
            _deviceId.value = service.deviceId()
            _platformStore.value = service.platformStore().toPrimitives()
            if (notificationsAvailable) {
                _pushToken.value = service.pushToken()
            }
        }
    }

    fun clearPendingTransactions() = launchAction { service.clearPendingTransactions() }

    fun clearTransactions() = launchAction { service.clearTransactions() }

    fun clearAssets() = launchAction { service.clearAssets() }

    fun clearDelegations() = launchAction { service.clearDelegations() }

    fun clearValidators() = launchAction { service.clearValidators() }

    fun clearBanners() = launchAction { service.clearBanners() }

    fun activateCancelledBanners() = launchAction { service.activateCancelledBanners() }

    fun clearPrices() = launchAction { service.clearPrices() }

    fun clearPerpetuals() = launchAction { service.clearPerpetualMarkets() }
    fun clearPreferences() = launchAction { service.clearPreferences() }
    fun resetTransactionsTimestamp() = launchWalletAction { service.resetTransactionsTimestamp(it.id) }
    fun deleteWalletPreferences() = launchWalletAction { service.deleteWalletPreferences(it.id) }
    fun addSampleTransactions() = launchWalletAction { service.addSampleTransactions(it.id) }

    private fun launchWalletAction(action: suspend (WalletId) -> Unit) = launchAction {
        val walletId = getSession().value?.wallet?.id ?: return@launchAction
        action(walletId)
    }

    private fun launchAction(action: suspend () -> Unit) {
        viewModelScope.launch(Dispatchers.IO) { action() }
    }
}
