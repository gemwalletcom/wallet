package com.gemwallet.android.features.settings.develop.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.cases.device.GetPushToken
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.wallet.core.primitives.PlatformStore
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class DevelopViewModel @Inject constructor(
    private val getDeviceId: GetDeviceId,
    private val getPushTokenCase: GetPushToken,
    private val clearPendingTransactions: ClearPendingTransactions,
    val platformStore: PlatformStore
) : ViewModel() {

    fun getDeviceId(): String {
        return getDeviceId.getDeviceId()
    }

    private val _pushToken = MutableStateFlow("")
    val pushToken = _pushToken.asStateFlow()

    init {
        viewModelScope.launch(Dispatchers.IO) {
            _pushToken.value = getPushTokenCase.getPushToken()
        }
    }

    fun resetTransactions() {
        viewModelScope.launch(Dispatchers.IO) {
            clearPendingTransactions.clearPending()
        }
    }
}