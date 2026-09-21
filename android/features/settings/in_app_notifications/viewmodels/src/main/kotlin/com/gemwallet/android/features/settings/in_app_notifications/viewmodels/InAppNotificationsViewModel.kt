package com.gemwallet.android.features.settings.in_app_notifications.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.notifications.cases.GetInAppNotifications
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.features.settings.in_app_notifications.viewmodels.models.NotificationRowUIModel
import com.gemwallet.android.features.settings.in_app_notifications.viewmodels.models.uiModel
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemNotificationServiceInterface
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class InAppNotificationsViewModel @Inject constructor(
    private val getCurrentWallet: GetCurrentWallet,
    private val getInAppNotifications: GetInAppNotifications,
    private val notificationService: GemNotificationServiceInterface,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val notifications: StateFlow<List<NotificationRowUIModel>> = getCurrentWallet.observe()
        .map { it?.id }
        .filterNotNull()
        .flatMapLatest { walletId -> getInAppNotifications(walletId) }
        .map { notifications -> notifications.map { it.uiModel(context) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    init {
        viewModelScope.launch {
            try {
                notificationService.open()
            } catch (err: Throwable) {
                Log.e(TAG, "Open notifications error", err)
            }
        }
    }

    companion object {
        private const val TAG = "InAppNotifications"
    }
}
