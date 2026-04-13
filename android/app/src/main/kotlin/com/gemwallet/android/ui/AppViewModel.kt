package com.gemwallet.android.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.BuildConfig
import com.gemwallet.android.application.assets.coordinators.GetWalletSummary
import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.cases.device.GetPushEnabled
import com.gemwallet.android.cases.device.SwitchPushEnabled
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.ext.VersionCheck
import com.gemwallet.android.features.onboarding.OnboardingDest
import com.gemwallet.android.model.Session
import com.gemwallet.android.ui.navigation.walletRootRoute
import com.wallet.core.primitives.PlatformStore
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

@HiltViewModel
class AppViewModel @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val userConfig: UserConfig,
    private val getPushEnabled: GetPushEnabled,
    private val switchPushEnabled: SwitchPushEnabled,
    private val walletsRepository: WalletsRepository,
    private val getRemoteConfig: GetRemoteConfig,
    getWalletSummary: GetWalletSummary,
) : ViewModel() {

    private val state = MutableStateFlow(AppState())
    val uiState = state.map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, AppUIState())
    private val startDestination = MutableStateFlow<String?>(null)
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
            walletRootRoute -> isWalletReady
            else -> true
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val isTermsAccepted = userConfig.isTermsAccepted()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val askNotifications = combine(
        userConfig.isAskNotifications(),
        sessionRepository.session(),
        getPushEnabled.getPushEnabled(),
    ) { isAsk, session, pushEnabled ->
        isAsk && session != null && !pushEnabled
    }.stateIn(viewModelScope, SharingStarted.Eagerly, false)


    init {
        viewModelScope.launch(Dispatchers.IO) {
            startDestination.value = getStartDestination()
        }
        viewModelScope.launch {
            handleAppVersion()
            rateAs()
            sessionRepository.session().collectLatest {
                onSession(it ?: return@collectLatest)
            }
        }
    }

    fun onSkip(version: String) = viewModelScope.launch {
        userConfig.setAppVersionSkip(version)
        state.update { it.copy(intent = AppIntent.None) }
    }

    fun onCancelUpdate() {
        state.update { it.copy(intent = AppIntent.None) }
    }

    private suspend fun handleAppVersion() = withContext(Dispatchers.IO) {
        if (BuildConfig.DEBUG) {
            return@withContext
        }
        val config = runCatching {
            getRemoteConfig.getRemoteConfig()
        }.getOrNull() ?: return@withContext

        val lastRelease = config.releases
            .filter {
                val versionFlavor = when (it.store) {
                    PlatformStore.GooglePlay -> "google"
                    PlatformStore.Fdroid -> "fdroid"
                    PlatformStore.Huawei -> "huawei"
                    PlatformStore.SolanaStore -> "solana"
                    PlatformStore.SamsungStore -> "samsung"
                    PlatformStore.ApkUniversal -> "universal"
                    PlatformStore.AppStore -> it.store.string
                    PlatformStore.Emerald -> "emerald"
                    PlatformStore.Local -> "local"
                }
                BuildConfig.FLAVOR == versionFlavor
            }
            .firstOrNull() ?: return@withContext

        val skipVersion = userConfig.getAppVersionSkip().firstOrNull()
        val lastVersion = lastRelease.version
        userConfig.setLatestVersion(lastVersion)
        if (VersionCheck.isVersionHigher(new = lastVersion, current = BuildConfig.VERSION_NAME) && skipVersion != lastVersion/* && current.store != PlatformStore.ApkUniversal*/) {
            state.update { it.copy(intent = AppIntent.ShowUpdate, version = lastVersion) }
        }
    }

    fun acceptTerms() {
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.acceptTerms()
        }
    }

    fun onNotificationsEnable() {
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.stopAskNotifications()
            switchPushEnabled.switchPushEnabled(
                true,
                walletsRepository.getAll().firstOrNull() ?: emptyList()
            )
        }
    }

    fun laterAskNotifications() {
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.stopAskNotifications()
        }
    }

    private fun rateAs() {
        if (userConfig.getLaunchNumber() == 10) {
            state.update { it.copy(intent = AppIntent.ShowReview) }
        }
        userConfig.increaseLaunchNumber()
    }

    private fun onSession(session: Session) {
        state.update {
            it.copy(session = session)
        }
    }

    private suspend fun getStartDestination(): String = withContext(Dispatchers.IO) {
        if (sessionRepository.getCurrentWallet() != null) {
            walletRootRoute
        } else {
            val wallet = walletsRepository.getAll().firstOrNull()
                ?.sortedWith(compareBy({ it.index }, { it.id }))
                ?.firstOrNull()
            if (wallet != null) {
                sessionRepository.setWallet(wallet)
                walletRootRoute
            } else {
                OnboardingDest.route
            }
        }
    }

    fun onReviewOpen() {
        state.update { it.copy(intent = AppIntent.None) }
    }
}

data class AppState(
    val session: Session? = null,
    val version: String = "",
    val intent: AppIntent = AppIntent.None,
) {
    fun toUIState() = AppUIState(
        session = session,
        version = version,
        intent = intent,
    )
}

data class AppUIState(
    val session: Session? = null,
    val version: String = "",
    val intent: AppIntent = AppIntent.None,
)

enum class AppIntent {
    None,
    ShowUpdate,
    ShowReview,
}
