package com.gemwallet.android.ui

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.PendingNavigationCoordinator
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.update.cases.SkipAppUpdate
import com.gemwallet.android.application.update.cases.SyncAppUpdate
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.Session
import com.gemwallet.android.ui.navigation.OnboardingRoute
import com.gemwallet.android.ui.navigation.WalletRootRoute
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAppStartServiceInterface
import uniffi.gemstone.GemAppUpdateOffer
import uniffi.gemstone.GemWalletSessionServiceInterface
import javax.inject.Inject

@HiltViewModel
class AppViewModel @Inject constructor(
    private val getSession: GetSession,
    private val userConfig: UserConfig,
    private val syncAppUpdate: SyncAppUpdate,
    private val skipAppUpdate: SkipAppUpdate,
    private val pendingNavigationCoordinator: PendingNavigationCoordinator,
    private val appStartService: GemAppStartServiceInterface,
    private val walletSessionService: GemWalletSessionServiceInterface,
    getWalletSummary: GetWalletSummary,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    fun openPayment(payload: String) {
        pendingNavigationCoordinator.pendScan(payload)
    }

    private val state = MutableStateFlow(AppState())
    val uiState = state.asStateFlow()
    private val startDestination = MutableStateFlow<NavKey?>(null)
    val startDestinationState = startDestination.asStateFlow()
    val session: StateFlow<Session?> = getSession()
    private val walletReadyState = getWalletSummary.getWalletSummary()
        .map { it != null }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)
    val launchReadyState = combine(
        startDestinationState,
        walletReadyState,
    ) { destination, isWalletReady ->
        when (destination) {
            null -> false
            WalletRootRoute -> isWalletReady
            else -> true
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val isTermsAccepted = userConfig.isTermsAccepted()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    init {
        viewModelScope.launch(ioDispatcher) {
            startDestination.value = getStartDestination()
        }
        viewModelScope.launch(ioDispatcher) {
            offerStoreUpdate()
            rateAs()
            getSession().collectLatest {
                onSession(it ?: return@collectLatest)
            }
        }
        viewModelScope.launch(ioDispatcher) {
            getSession()
                .filterNotNull()
                .distinctUntilChangedBy { it.wallet.id }
                .collectLatest { session ->
                    appStartService.setupWallet(session.wallet.toGem()).forEach { failure ->
                        Log.e(TAG, "${failure.step} failed: ${failure.message}")
                    }
                }
        }
    }

    fun onSkip() = viewModelScope.launch {
        val update = state.value.update ?: return@launch
        runCatchingCancellable { skipAppUpdate.skipAppUpdate(update) }
            .onSuccess { state.update { it.copy(update = null) } }
            .onFailure { Log.e(TAG, "skipping update ${update.version} failed", it) }
    }

    fun onUpdateOpened() {
        if (state.value.update?.canSkip() == false) {
            return
        }
        state.update { it.copy(update = null) }
    }

    private suspend fun offerStoreUpdate() {
        val offer = syncAppUpdate.syncAppUpdate() ?: return
        if (offer.apkUrl != null) {
            return
        }
        state.update { it.copy(update = offer) }
    }

    fun acceptTerms() {
        viewModelScope.launch(ioDispatcher) {
            userConfig.acceptTerms()
        }
    }

    private fun rateAs() {
        userConfig.increaseLaunchNumber()
        if (!userConfig.shouldRequestReview()) {
            return
        }
        state.update { it.copy(intent = AppIntent.ShowReview) }
        userConfig.setRateApplicationShown()
    }

    private fun onSession(session: Session) {
        state.update {
            it.copy(session = session)
        }
    }

    private suspend fun getStartDestination(): NavKey = withContext(ioDispatcher) {
        when (walletSessionService.ensureCurrentWallet()) {
            null -> OnboardingRoute
            else -> WalletRootRoute
        }
    }

    fun onReviewOpen() {
        state.update { it.copy(intent = AppIntent.None) }
    }

    private companion object {
        const val TAG = "App"
    }
}

data class AppState(val session: Session? = null, val intent: AppIntent = AppIntent.None, val update: GemAppUpdateOffer? = null)

enum class AppIntent {
    None,
    ShowReview,
}
