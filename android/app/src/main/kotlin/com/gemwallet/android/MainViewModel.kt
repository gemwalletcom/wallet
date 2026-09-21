package com.gemwallet.android

import android.content.Context
import android.content.Intent
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_connect.cases.IsWalletConnectEnabled
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.pricealerts.MigratePriceAlertsPreference
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.model.AuthState
import com.gemwallet.android.services.MigrateV3KeystoreService
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Appearance
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAppStartFailure
import uniffi.gemstone.GemAppStartServiceInterface
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemPaymentException
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemWalletServiceInterface
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val isWalletConnectEnabledCase: IsWalletConnectEnabled,
    private val pairWalletConnect: PairWalletConnect,
    private val appStartService: GemAppStartServiceInterface,
    private val migrateV3KeystoreService: MigrateV3KeystoreService,
    private val walletService: GemWalletServiceInterface,
    private val migratePriceAlertsPreference: MigratePriceAlertsPreference,
    private val lockTimer: LockTimer,
    private val pendingNavigationCoordinator: PendingNavigationCoordinator,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val isInitialAuthRequired = userConfig.authRequired()

    private val _uiState = MutableStateFlow(
        MainUIState(
            initialAuth = if (isInitialAuthRequired) AuthState.Required else AuthState.Success,
            hasUnlockedApp = !isInitialAuthRequired,
        ),
    )
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
                _uiState.map { it.initialAuth == AuthState.Success }.distinctUntilChanged(),
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
                                error.errorText?.let { state.copy(navigationError = it) }
                                    ?: state.copy(isScanErrorVisible = true)
                            }
                        }
                    } catch (error: GemServiceException) {
                        val input = when (val pending = pendingNavigationCoordinator.pendingNavigation.value) {
                            is PendingNavigation.Loading -> pending.input
                            else -> pending as? PendingNavigation.Input
                        }
                        pendingNavigationCoordinator.clear()
                        when (input?.code) {
                            null -> Log.e("MainViewModel", "notification navigation failed", error)
                            else -> _uiState.update { it.copy(navigationError = error.errorText()) }
                        }
                    }
                }
        }
    }

    fun isAuthRequired(): Boolean = userConfig.authRequired()

    private var isMaintained = false

    internal fun maintain() {
        if (isMaintained) return
        isMaintained = true
        viewModelScope.launch(ioDispatcher) { appStartService.run().forEach(::logAppStartFailure) }
        viewModelScope.launch(ioDispatcher) {
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
        viewModelScope.launch(ioDispatcher) {
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

    fun pendIntent(intent: Intent) = pendingNavigationCoordinator.pendIntent(intent)

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
        viewModelScope.launch(ioDispatcher) {
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
        val initialAuth: AuthState = AuthState.Required,
        val authState: AuthState? = null,
        val authPromptRequest: Int = 0,
        val hasUnlockedApp: Boolean = false,
        val isWalletConnectPairingToastVisible: Boolean = false,
        val walletConnectError: String? = null,
        val navigationError: GemErrorText? = null,
        val isWalletConnectUnsupportedVisible: Boolean = false,
        val isScanErrorVisible: Boolean = false,
    )
}

private const val NoActiveAuthRequestId = -1L
