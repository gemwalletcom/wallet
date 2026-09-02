package com.gemwallet.android.features.settings.in_app_notifications.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.notifications.cases.GetInAppNotifications
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.wallet.core.primitives.InAppNotification
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject
import uniffi.gemstone.GemNotificationService

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class InAppNotificationsViewModel @Inject constructor(
    private val getCurrentWallet: GetCurrentWallet,
    private val getInAppNotifications: GetInAppNotifications,
    private val notificationService: GemNotificationService,
) : ViewModel() {

    val notifications: StateFlow<List<InAppNotification>> = getCurrentWallet.observe()
        .map { it?.id }
        .filterNotNull()
        .flatMapLatest { walletId -> getInAppNotifications(walletId) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    init {
        viewModelScope.launch {
            val wallet = getCurrentWallet.getCurrentWallet() ?: return@launch
            try {
                notificationService.open(wallet.id.id)
            } catch (err: Throwable) {
                Log.e(TAG, "Open notifications error", err)
            }
        }
    }

    companion object {
        private const val TAG = "InAppNotifications"
    }
}
