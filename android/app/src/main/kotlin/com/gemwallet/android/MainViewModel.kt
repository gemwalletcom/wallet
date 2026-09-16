package com.gemwallet.android

import android.content.Intent
import android.os.SystemClock
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.IsWalletConnectEnabled
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.pricealerts.MigratePriceAlertsPreference
import com.gemwallet.android.ext.userMessage
import com.gemwallet.android.model.AuthState
import android.util.Log
import com.gemwallet.android.services.MigrateV3KeystoreService
import uniffi.gemstone.GemWalletService
import com.wallet.core.primitives.Appearance
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAppLockPhase
import uniffi.gemstone.GemAppLockSession
import uniffi.gemstone.GemAppLockSettings
import uniffi.gemstone.GemAppStartFailure
import uniffi.gemstone.GemAppStartServiceInterface
import uniffi.gemstone.GemAuthPromptOutcome
import uniffi.gemstone.GemPaymentException
import uniffi.gemstone.lockPeriodFromMinutes
import uniffi.gemstone.newAppLockSession
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val isWalletConnectEnabledCase: IsWalletConnectEnabled,
    private val pairWalletConnect: PairWalletConnect,
    private val appStartService: GemAppStartServiceInterface,
    private val migrateV3KeystoreService: MigrateV3KeystoreService,
    private val walletService: GemWalletService,
    private val migratePriceAlertsPreference: MigratePriceAlertsPreference,
    private val activeWalletConnectRequest: ActiveWalletConnectRequest,
    private val pendingNavigationCoordinator: PendingNavigationCoordinator,
) : ViewModel() {

    private val _uiState = MutableStateFlow(MainUIState(lock = newAppLockSession(lockSettings(lockPeriodMinutes = null))))
    val uiState: StateFlow<MainUIState> = _uiState.asStateFlow()

    internal val pendingNavigation: StateFlow<PendingNavigation?> = pendingNavigationCoordinator.pendingNavigation

    private val activeAuthRequestId = AtomicLong(NoActiveAuthRequestId)

    val isWalletConnectEnabled: Boolean = isWalletConnectEnabledCase.isWalletConnectEnabled()

    val appearance = userConfig.appearance()
        .stateIn(viewModelScope, SharingStarted.Eagerly, Appearance.System)

    private val walletConnectHandler = object : PendingNavigationCoordinator.WalletConnectHandler {
        override fun onPairing(uri: String) = addPairing(uri)
        override fun onRequest() {
            if (isWalletConnectEnabled) {
                showWalletConnectPairingToast()
            } else {
                showWalletConnectUnsupported()
            }
        }
    }

    init {
        viewModelScope.launch {
            combine(
                _uiState.map { it.lock.phase == GemAppLockPhase.Unlocked }.distinctUntilChanged(),
                pendingNavigation,
            ) { unlocked, pending -> unlocked && pending is PendingNavigation.Input }
                .distinctUntilChanged()
                .filter { it }
                .collect {
                    try {
                        val handled = pendingNavigationCoordinator.buildRoutes(walletConnectHandler)
                        if (!handled) {
                            _uiState.update { it.copy(isScanErrorVisible = true) }
                        }
                    } catch (error: GemPaymentException) {
                        val isLoadingPayment = pendingNavigationCoordinator.pendingNavigation.value is PendingNavigation.Loading
                        pendingNavigationCoordinator.clear()
                        if (isLoadingPayment) {
                            _uiState.update { state ->
                                error.userMessage?.let { state.copy(navigationError = it) }
                                    ?: state.copy(isScanErrorVisible = true)
                            }
                        }
                    }
                }
        }
    }

    fun isAuthRequired(): Boolean = userConfig.authRequired()

    fun isUnlocked(): Boolean = uiState.value.lock.phase == GemAppLockPhase.Unlocked

    internal fun maintain() {
        viewModelScope.launch(Dispatchers.IO) { appStartService.run().forEach(::logAppStartFailure) }
        viewModelScope.launch(Dispatchers.IO) {
            migratePriceAlertsPreference()
            migrateV3KeystoreService()
            runCatching { walletService.migrateToSharedPassword() }
                .onFailure { Log.e("MainViewModel", "shared keystore password migration failed", it) }
            appStartService.setupWallets().forEach(::logAppStartFailure)
        }
    }

    private fun logAppStartFailure(failure: GemAppStartFailure) {
        Log.e("MainViewModel", "${failure.step} failed: ${failure.message}")
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

    fun onUnlockRequested() = updateLock { it.onUnlockRequested() }

    fun onUnlocked() = updateLock { lock ->
        (lock.phase as? GemAppLockPhase.Unlocking)?.let { lock.onUnlocked(it.attempt) } ?: lock
    }

    fun onUnlockFailed(outcome: GemAuthPromptOutcome) = updateLock { lock ->
        (lock.phase as? GemAppLockPhase.Unlocking)?.let { lock.onUnlockFailed(it.attempt, outcome) } ?: lock
    }

    fun completeAuthRequest(requestId: Long): Boolean {
        if (!activeAuthRequestId.compareAndSet(requestId, NoActiveAuthRequestId)) return false
        _uiState.update { it.copy(authState = null) }
        return true
    }

    fun onActivityPaused() = pause(now = SystemClock.elapsedRealtime())

    fun onActivityResumed() {
        viewModelScope.launch(Dispatchers.IO) {
            val settings = lockSettings(lockPeriodMinutes = userConfig.getLockInterval().first())
            resume(settings, hasPendingRequest = activeWalletConnectRequest.current.value != null, now = SystemClock.elapsedRealtime())
        }
    }

    internal fun pause(now: Long) = updateLock { it.onBackground(now) }

    internal fun resume(settings: GemAppLockSettings, hasPendingRequest: Boolean, now: Long) {
        val wasUnlocked = isUnlocked()
        updateLock { it.onSettingsChanged(settings).onActive(now, hasPendingRequest) }
        if (wasUnlocked && !isUnlocked()) {
            activeAuthRequestId.set(NoActiveAuthRequestId)
            _uiState.update { it.copy(authState = null) }
        }
    }

    private fun updateLock(transition: (GemAppLockSession) -> GemAppLockSession) {
        _uiState.update { it.copy(lock = transition(it.lock)) }
    }

    private fun lockSettings(lockPeriodMinutes: Int?) = GemAppLockSettings(
        authenticationRequired = userConfig.authRequired(),
        privacyLockEnabled = false,
        lockPeriod = lockPeriodFromMinutes(lockPeriodMinutes?.toUInt()),
    )

    fun handleIntent(intent: Intent) = pendingNavigationCoordinator.handleIntent(intent)

    fun consumePendingNavigation() = pendingNavigationCoordinator.clear()

    fun dismissScanError() {
        _uiState.update { it.copy(isScanErrorVisible = false) }
    }

    fun dismissWalletConnectPairingToast() {
        _uiState.update { it.copy(isWalletConnectPairingToastVisible = false) }
    }

    fun resetError() {
        _uiState.update {
            it.copy(
                navigationError = null,
                walletConnectError = null,
                isWalletConnectUnsupportedVisible = false,
            )
        }
    }

    fun showWalletConnectError(error: String) {
        _uiState.update { it.copy(walletConnectError = error) }
    }

    private fun addPairing(uri: String) {
        if (!isWalletConnectEnabled) {
            showWalletConnectUnsupported()
            return
        }
        showWalletConnectPairingToast()
        viewModelScope.launch(Dispatchers.IO) {
            pairWalletConnect.pair(
                uri = uri,
                onSuccess = {},
                onError = ::showWalletConnectError,
            )
        }
    }

    private fun showWalletConnectPairingToast() {
        _uiState.update { it.copy(isWalletConnectPairingToastVisible = true) }
    }

    private fun showWalletConnectUnsupported() {
        _uiState.update { it.copy(isWalletConnectUnsupportedVisible = true) }
    }

    data class MainUIState(
        val lock: GemAppLockSession,
        val authState: AuthState? = null,
        val authPromptRequest: Int = 0,
        val isWalletConnectPairingToastVisible: Boolean = false,
        val walletConnectError: String? = null,
        val navigationError: String? = null,
        val isWalletConnectUnsupportedVisible: Boolean = false,
        val isScanErrorVisible: Boolean = false,
    )
}

private const val NoActiveAuthRequestId = -1L
