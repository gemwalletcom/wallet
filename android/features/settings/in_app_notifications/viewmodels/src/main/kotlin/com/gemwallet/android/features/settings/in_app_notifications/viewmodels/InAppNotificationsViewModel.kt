package com.gemwallet.android.features.settings.in_app_notifications.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.notifications.InAppNotificationsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.InAppNotification
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class InAppNotificationsViewModel @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val repository: InAppNotificationsRepository,
) : ViewModel() {

    val notifications: StateFlow<List<InAppNotification>> = sessionRepository.session()
        .map { it?.wallet?.id }
        .filterNotNull()
        .flatMapLatest { walletId -> repository.getNotifications(walletId) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    init {
        viewModelScope.launch {
            val wallet = sessionRepository.getCurrentWallet() ?: return@launch
            try {
                repository.sync(wallet.id)
            } catch (err: Throwable) {
                Log.e(TAG, "Sync notifications error", err)
            }
            try {
                val hasUnread = repository.getNotifications(wallet.id).first().any { it.readAt == null }
                if (hasUnread) {
                    repository.markNotificationsRead()
                }
            } catch (err: Throwable) {
                Log.e(TAG, "Mark notifications read error", err)
            }
        }
    }

    companion object {
        private const val TAG = "InAppNotifications"
    }
}
