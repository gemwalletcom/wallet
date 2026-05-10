package com.gemwallet.android

import android.content.Intent
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.model.AuthState
import com.gemwallet.android.services.CheckAccountsService
import com.gemwallet.android.services.SyncService
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
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
    private val lockTimer: LockTimer,
    private val pendingNavigationCoordinator: PendingNavigationCoordinator,
) : ViewModel() {

    private val isInitialAuthRequired = userConfig.authRequired()

    private val _uiState = MutableStateFlow(
        MainUIState(
            initialAuth = if (isInitialAuthRequired) AuthState.Required else AuthState.Success,
            hasUnlockedApp = !isInitialAuthRequired,
        )
    )
    val uiState: StateFlow<MainUIState> = _uiState.asStateFlow()

    internal val pendingNavigation: StateFlow<PendingNavigation?> = pendingNavigationCoordinator.pendingNavigation

    private val activeAuthRequestId = AtomicLong(NoActiveAuthRequestId)

    private val walletConnectHandler = object : PendingNavigationCoordinator.WalletConnectHandler {
        override fun onPairing(uri: String) = addPairing(uri)
        override fun onRequest() = showWalletConnectPairingToast()
    }

    init {
        viewModelScope.launch {
            combine(
                _uiState.map { it.initialAuth == AuthState.Success }.distinctUntilChanged(),
                pendingNavigation,
            ) { unlocked, pending -> unlocked && pending is PendingNavigation.RawIntent }
                .distinctUntilChanged()
                .filter { it }
                .collect {
                    pendingNavigationCoordinator.resolve(walletConnectHandler)
                }
        }
    }

    fun isAuthRequired(): Boolean = userConfig.authRequired()

    internal fun maintain() {
        viewModelScope.launch(Dispatchers.IO) { syncService.sync() }
        viewModelScope.launch(Dispatchers.IO) { checkAccountsService() }
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
                current.copy(
                    initialAuth = authState,
                    hasUnlockedApp = current.hasUnlockedApp || authState == AuthState.Success,
                )
            }
        }
    }

    fun completeAuthRequest(requestId: Long): Boolean {
        if (!activeAuthRequestId.compareAndSet(requestId, NoActiveAuthRequestId)) return false
        _uiState.update { it.copy(authState = null) }
        return true
    }

    fun onActivityPaused() {
        lockTimer.onPaused()
    }

    fun onActivityResumed() {
        viewModelScope.launch(Dispatchers.IO) {
            if (lockTimer.shouldRelock()) relock()
        }
    }

    internal fun relock() {
        activeAuthRequestId.set(NoActiveAuthRequestId)
        _uiState.update { current ->
            current.copy(
                initialAuth = AuthState.Required,
                authState = null,
                authPromptRequest = current.authPromptRequest + 1,
            )
        }
    }

    fun handleIntent(intent: Intent) = pendingNavigationCoordinator.handleIntent(intent)

    fun consumePendingNavigation() = pendingNavigationCoordinator.consume()

    fun dismissWalletConnectPairingToast() {
        _uiState.update { it.copy(isWalletConnectPairingToastVisible = false) }
    }

    fun resetWalletConnectError() {
        _uiState.update { it.copy(walletConnectError = null) }
    }

    private fun addPairing(uri: String) {
        showWalletConnectPairingToast()
        viewModelScope.launch(Dispatchers.IO) {
            bridgesRepository.addPairing(
                uri = uri,
                onSuccess = {},
                onError = { error -> _uiState.update { it.copy(walletConnectError = error) } },
            )
        }
    }

    private fun showWalletConnectPairingToast() {
        _uiState.update { it.copy(isWalletConnectPairingToastVisible = true) }
    }

    data class MainUIState(
        val initialAuth: AuthState = AuthState.Required,
        val authState: AuthState? = null,
        val authPromptRequest: Int = 0,
        val hasUnlockedApp: Boolean = false,
        val isWalletConnectPairingToastVisible: Boolean = false,
        val walletConnectError: String? = null,
    )
}

private const val NoActiveAuthRequestId = -1L
