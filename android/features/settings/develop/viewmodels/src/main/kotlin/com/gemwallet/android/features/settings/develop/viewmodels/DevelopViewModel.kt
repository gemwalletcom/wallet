package com.gemwallet.android.features.settings.develop.viewmodels

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
            _platformStore.value = service.platformStore().decodeJson<PlatformStore>()
            if (notificationsAvailable) {
                _pushToken.value = service.pushToken()
            }
        }
    }

    fun resetTransactions() {
        viewModelScope.launch(Dispatchers.IO) {
            service.clearPendingTransactions()
        }
    }
}
