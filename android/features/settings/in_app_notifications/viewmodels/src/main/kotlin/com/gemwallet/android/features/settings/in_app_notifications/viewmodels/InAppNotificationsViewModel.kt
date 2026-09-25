package com.gemwallet.android.features.settings.in_app_notifications.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.InAppNotificationsQuery
import com.gemwallet.android.features.settings.in_app_notifications.viewmodels.models.NotificationRowUIModel
import com.gemwallet.android.features.settings.in_app_notifications.viewmodels.models.uiModels
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemNotificationServiceInterface
import uniffi.gemstone.loadError
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class InAppNotificationsViewModel @Inject constructor(
    getCurrentWalletId: GetCurrentWalletId,
    private val inAppNotificationsQuery: InAppNotificationsQuery,
    private val notificationService: GemNotificationServiceInterface,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val loadState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    val notifications: StateFlow<List<NotificationRowUIModel>> = getCurrentWalletId()
        .flatMapLatest { walletId -> inAppNotificationsQuery(walletId.id) }
        .map { notifications -> notifications.uiModels(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val errorRow: StateFlow<GemListRow?> = combine(loadState, notifications) { state, items ->
        loadError(state, items.isNotEmpty())?.let { GemListRow.Error(it) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    init {
        viewModelScope.launch {
            loadState.update { notificationService.refresh(notifications.value.isNotEmpty()) }
        }
    }
}
