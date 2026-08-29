package com.gemwallet.android.features.settings.develop.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.device.cases.GetPushToken
import com.gemwallet.android.application.transactions.cases.ClearPendingTransactions
import com.gemwallet.android.model.NotificationsAvailable
import com.wallet.core.primitives.PlatformStore
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject
import uniffi.gemstone.GemDeviceKeyService

@HiltViewModel
class DevelopViewModel @Inject constructor(
    private val deviceKeyService: GemDeviceKeyService,
    private val getPushTokenCase: GetPushToken,
    private val clearPendingTransactions: ClearPendingTransactions,
    val platformStore: PlatformStore,
    val notificationsAvailable: NotificationsAvailable,
) : ViewModel() {

    private val _deviceId = MutableStateFlow("")
    val deviceId = _deviceId.asStateFlow()
    private val _pushToken = MutableStateFlow("")
    val pushToken = _pushToken.asStateFlow()

    init {
        viewModelScope.launch(Dispatchers.IO) {
            _deviceId.value = deviceKeyService.deviceId()
            if (notificationsAvailable) {
                _pushToken.value = getPushTokenCase.getPushToken()
            }
        }
    }

    fun resetTransactions() {
        viewModelScope.launch(Dispatchers.IO) {
            clearPendingTransactions.clearPending()
        }
    }
}
