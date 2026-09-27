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
import com.gemwallet.android.services.MigrateV3KeystoreService
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Appearance
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAppStartFailure
import uniffi.gemstone.GemAppStartServiceInterface
import uniffi.gemstone.GemPaymentException
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemWalletServiceInterface
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
    private val pendingNavigationCoordinator: PendingNavigationCoordinator,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val _uiState = MutableStateFlow(MainUIState())
    val uiState: StateFlow<MainUIState> = _uiState.asStateFlow()

    internal val pendingNavigation: StateFlow<PendingNavigation?> = pendingNavigationCoordinator.pendingNavigation

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

    private var isMaintained = false

    internal fun maintain(isUnlocked: Flow<Boolean>) {
        if (isMaintained) return
        isMaintained = true
        viewModelScope.launch {
            combine(
                isUnlocked.distinctUntilChanged(),
                pendingNavigation,
            ) { unlocked, pending -> unlocked && pending is PendingNavigation.Input }
                .distinctUntilChanged()
                .filter { it }
                .collect {
                    try {
                        pendingNavigationCoordinator.buildRoutes(walletConnectHandler)?.let { text ->
                            _uiState.update { it.copy(navigationError = text.text(context)) }
                        }
                    } catch (error: GemPaymentException) {
                        onNavigationFailed(error)
                    } catch (error: GemServiceException) {
                        onNavigationFailed(error)
                    }
                }
        }
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

    fun pendIntent(intent: Intent) = pendingNavigationCoordinator.pendIntent(intent)

    fun consumePendingNavigation() = pendingNavigationCoordinator.clear()

    private fun onNavigationFailed(error: Exception) {
        val input = when (val pending = pendingNavigationCoordinator.pendingNavigation.value) {
            is PendingNavigation.Loading -> pending.input
            else -> pending as? PendingNavigation.Input
        }
        pendingNavigationCoordinator.clear()
        when (input?.code) {
            null -> Log.e("MainViewModel", "notification navigation failed", error)
            else -> _uiState.update { it.copy(navigationError = error.errorText().text(context)) }
        }
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
                onError = { showWalletConnectError(it.text(context)) },
            )
        }
    }

    private fun showWalletConnectPairingToast() {
        _uiState.update { it.copy(isWalletConnectPairingToastVisible = true) }
    }

    private fun showWalletConnectUnsupported() {
        _uiState.update { it.copy(isWalletConnectUnsupportedVisible = true) }
    }

    data class MainUIState(val isWalletConnectPairingToastVisible: Boolean = false, val walletConnectError: String? = null, val navigationError: String? = null, val isWalletConnectUnsupportedVisible: Boolean = false)
}
