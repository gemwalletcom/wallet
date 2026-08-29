package com.gemwallet.android.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.update.cases.SkipAppUpdate
import com.gemwallet.android.application.update.cases.SyncAppUpdate
import com.gemwallet.android.application.wallet_import.cases.SetupWallet
import com.gemwallet.android.application.device.cases.GetPushEnabled
import com.gemwallet.android.application.device.cases.SwitchPushEnabled
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.model.AppUpdateChannel
import com.gemwallet.android.model.AppUpdateOffer
import com.gemwallet.android.application.session.cases.GetCurrentWallet
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.application.wallet.cases.GetWallets
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.onboarding.OnboardingRoute
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.NotificationsAvailable
import com.gemwallet.android.PendingNavigationCoordinator
import com.gemwallet.android.ui.navigation.WalletRootRoute
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

@HiltViewModel
class AppViewModel @Inject constructor(
    private val getSession: GetSession,
    private val getCurrentWallet: GetCurrentWallet,
    private val setCurrentWallet: SetCurrentWallet,
    private val userConfig: UserConfig,
    private val getPushEnabled: GetPushEnabled,
    private val switchPushEnabled: SwitchPushEnabled,
    private val getWallets: GetWallets,
    private val syncAppUpdate: SyncAppUpdate,
    private val skipAppUpdate: SkipAppUpdate,
    private val notificationsAvailable: NotificationsAvailable,
    private val pendingNavigationCoordinator: PendingNavigationCoordinator,
    private val setupWallet: SetupWallet,
    getWalletSummary: GetWalletSummary,
) : ViewModel() {

    fun openPayment(payload: String) {
        pendingNavigationCoordinator.handleScan(payload)
    }

    private val state = MutableStateFlow(AppState())
    val uiState = state.asStateFlow()
    private val startDestination = MutableStateFlow<NavKey?>(null)
    val startDestinationState = startDestination.asStateFlow()
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

    val askNotifications = combine(
        userConfig.isAskNotifications(),
        getSession(),
        getPushEnabled.getPushEnabled(),
    ) { isAsk, session, pushEnabled ->
        notificationsAvailable && isAsk && session != null && !pushEnabled
    }.stateIn(viewModelScope, SharingStarted.Eagerly, false)

    init {
        viewModelScope.launch(Dispatchers.IO) {
            startDestination.value = getStartDestination()
        }
        viewModelScope.launch(Dispatchers.IO) {
            handleAppVersion()
            rateAs()
            getSession().collectLatest {
                onSession(it ?: return@collectLatest)
            }
        }
        viewModelScope.launch(Dispatchers.IO) {
            getSession()
                .filterNotNull()
                .distinctUntilChangedBy { it.wallet.id }
                .collectLatest { setupWallet.setup(it.wallet) }
        }
    }

    fun onSkip() = viewModelScope.launch {
        val update = state.value.update ?: return@launch
        if (update.isRequired) {
            return@launch
        }
        skipAppUpdate.skipAppUpdate(update.version)
        state.update { it.copy(update = null) }
    }

    fun onCancelUpdate() {
        if (state.value.update?.isRequired == true) {
            return
        }
        state.update { it.copy(update = null) }
    }

    private suspend fun handleAppVersion() {
        val offer = syncAppUpdate.syncAppUpdate() ?: return
        if (offer.channel != AppUpdateChannel.Store) {
            return
        }
        state.update { it.copy(update = offer) }
    }

    fun acceptTerms() {
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.acceptTerms()
        }
    }

    fun onNotificationsEnable() {
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.stopAskNotifications()
            switchPushEnabled.switchPushEnabled(true)
        }
    }

    fun laterAskNotifications() {
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.stopAskNotifications()
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

    private suspend fun getStartDestination(): NavKey = withContext(Dispatchers.IO) {
        if (getCurrentWallet.getCurrentWallet() != null) {
            WalletRootRoute
        } else {
            val wallet = getWallets().firstOrNull()
                ?.filter { it.accounts.isNotEmpty() }
                ?.sortedWith(compareBy({ it.index }, { it.id.id }))
                ?.firstOrNull()
            if (wallet != null) {
                setCurrentWallet.setCurrentWallet(wallet.id)
                WalletRootRoute
            } else {
                OnboardingRoute
            }
        }
    }

    fun onReviewOpen() {
        state.update { it.copy(intent = AppIntent.None) }
    }
}

data class AppState(
    val session: Session? = null,
    val intent: AppIntent = AppIntent.None,
    val update: AppUpdateOffer? = null,
)

enum class AppIntent {
    None,
    ShowReview,
}
