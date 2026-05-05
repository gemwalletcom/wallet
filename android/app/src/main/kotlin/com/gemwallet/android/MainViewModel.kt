package com.gemwallet.android

import android.content.Intent
import android.os.SystemClock
import android.text.format.DateUtils
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.model.AuthState
import com.gemwallet.android.services.CheckAccountsService
import com.gemwallet.android.services.SyncService
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val bridgesRepository: BridgesRepository,
    private val syncService: SyncService,
    private val checkAccountsService: CheckAccountsService,
    private val notificationNavigation: NotificationNavigation,
) : ViewModel() {

    private val _pendingNavigation = MutableStateFlow<PendingNavigation?>(null)
    internal val pendingNavigation = _pendingNavigation.asStateFlow()

    private val _uiState = MutableStateFlow(
        MainUIState(
            initialAuth = if (userConfig.authRequired()) AuthState.Required else AuthState.Success
        )
    )

    private val pauseTime = AtomicLong(0)
    private val activeAuthRequestId = AtomicLong(NoActiveAuthRequestId)

    val uiState = _uiState.asStateFlow()

    fun isAuthRequired(request: AuthRequest): Boolean =
        request == AuthRequest.Enable || userConfig.authRequired()

    internal fun maintain() {
        viewModelScope.launch(Dispatchers.IO) {
            syncService.sync()
        }
        viewModelScope.launch(Dispatchers.IO) {
            checkAccountsService()
        }
    }

    fun requestAuth(requestId: Long) {
        activeAuthRequestId.set(requestId)
        _uiState.update { current ->
            current.copy(
                authState = AuthState.Required,
                authPromptRequest = current.authPromptRequest + 1,
            )
        }
    }

    fun retryInitialAuth() {
        _uiState.update { current ->
            if (current.initialAuth == AuthState.Success) {
                current
            } else {
                current.copy(
                    initialAuth = AuthState.Required,
                    authPromptRequest = current.authPromptRequest + 1,
                )
            }
        }
    }

    fun onInitialAuth(authState: AuthState) {
        _uiState.update { current ->
            if (current.initialAuth == AuthState.Success) {
                current
            } else {
                current.copy(initialAuth = authState)
            }
        }
    }

    fun completeAuthRequest(requestId: Long): Boolean {
        if (!activeAuthRequestId.compareAndSet(requestId, NoActiveAuthRequestId)) return false
        _uiState.update { it.copy(authState = null) }
        return true
    }

    private fun addPairing(uri: String) {
        showWalletConnectPairingToast()
        viewModelScope.launch(Dispatchers.IO) {
            bridgesRepository.addPairing(
                uri = uri,
                onSuccess = {},
                onError = { error ->
                    _uiState.update {
                        it.copy(walletConnectError = error)
                    }
                }
            )
        }
    }

    private fun showWalletConnectPairingToast() {
        _uiState.update { it.copy(isWalletConnectPairingToastVisible = true) }
    }

    fun dismissWalletConnectPairingToast() {
        _uiState.update { it.copy(isWalletConnectPairingToastVisible = false) }
    }

    fun resetWalletConnectError() {
        _uiState.update { it.copy(walletConnectError = null) }
    }

    fun onActivityResumed() {
        viewModelScope.launch(Dispatchers.IO) {
            if (!userConfig.authRequired()) return@launch

            val interval = SystemClock.elapsedRealtime() - pauseTime.get()
            val lockInterval = (userConfig.getLockInterval().firstOrNull() ?: 0) * DateUtils.MINUTE_IN_MILLIS
            if (interval > lockInterval) {
                _uiState.update { it.copy(initialAuth = AuthState.Required) }
            }
        }
    }

    fun onActivityPaused() {
        pauseTime.set(SystemClock.elapsedRealtime())
    }

    fun handleIntent(intent: Intent) {
        if (intent.hasNotificationPayload() || intent.dataString != null) {
            _pendingNavigation.update { PendingNavigation.RawIntent(Intent(intent)) }
        }
    }

    fun preparePendingNavigation() {
        viewModelScope.launch(Dispatchers.IO) {
            val pendingIntent = (_pendingNavigation.value as? PendingNavigation.RawIntent)?.intent ?: return@launch
            val uri = pendingIntent.dataString
            if (handleWalletConnect(uri)) {
                clearPendingIntent(pendingIntent)
                return@launch
            }

            uri?.toWebDeepLinkRoute()?.let { route ->
                setPendingRoute(pendingIntent, route)
                return@launch
            }

            if (!pendingIntent.hasNotificationPayload()) {
                clearPendingIntent(pendingIntent)
                return@launch
            }

            val pendingRoute = notificationNavigation.prepareNavigation(pendingIntent)
            if (pendingRoute == null) {
                clearPendingIntent(pendingIntent)
                return@launch
            }

            setPendingRoute(pendingIntent, pendingRoute)
        }
    }

    private fun handleWalletConnect(uri: String?): Boolean {
        return when (val link = uri?.toWalletConnectLink() ?: return false) {
            is WalletConnectLink.Pairing -> {
                addPairing(link.uri)
                true
            }
            WalletConnectLink.Request -> {
                showWalletConnectPairingToast()
                true
            }
            WalletConnectLink.Session -> true
        }
    }

    fun consumePendingNavigation() {
        _pendingNavigation.update { null }
    }

    private fun clearPendingIntent(consumedIntent: Intent) {
        updatePendingIntent(consumedIntent, replacement = null)
    }

    private fun setPendingRoute(pendingIntent: Intent, route: NavKey) {
        updatePendingIntent(pendingIntent, replacement = PendingNavigation.Route(route))
    }

    private fun updatePendingIntent(pendingIntent: Intent, replacement: PendingNavigation?) {
        _pendingNavigation.update { current ->
            if (current is PendingNavigation.RawIntent && current.intent === pendingIntent) {
                replacement
            } else {
                current
            }
        }
    }

    data class MainUIState(
        val initialAuth: AuthState = AuthState.Required,
        val authState: AuthState? = null,
        val authPromptRequest: Int = 0,
        val isWalletConnectPairingToastVisible: Boolean = false,
        val walletConnectError: String? = null,
    )
}

internal sealed interface PendingNavigation {
    data class RawIntent(val intent: Intent) : PendingNavigation
    data class Route(val route: NavKey) : PendingNavigation
}

private const val NoActiveAuthRequestId = -1L
