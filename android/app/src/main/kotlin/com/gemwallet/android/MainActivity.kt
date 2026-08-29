package com.gemwallet.android

import android.content.Intent
import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.core.view.WindowCompat
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.cases.security.AuthRequester
import com.gemwallet.android.application.bridge.ActiveWalletConnectRequest
import com.gemwallet.android.data.repositories.connection.ConnectionStatusObserver
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.AppViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.ConnectionBannerState
import com.gemwallet.android.ui.components.LocalConnectionBannerState
import com.wallet.core.primitives.Appearance
import com.wallet.core.primitives.ConnectionStatus
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : FragmentActivity(), AuthRequester {
    private val viewModel: MainViewModel by viewModels()
    private val appViewModel: AppViewModel by viewModels()
    private lateinit var systemAuthenticator: SystemAuthenticator

    @Inject lateinit var connectionStatusObserver: ConnectionStatusObserver
    @Inject lateinit var activeWalletConnectRequest: ActiveWalletConnectRequest

    override fun onCreate(savedInstanceState: Bundle?) {
        val splashScreen = installSplashScreen()
        super.onCreate(savedInstanceState)
        splashScreen.setKeepOnScreenCondition { !appViewModel.launchReadyState.value }
        splashScreen.setOnExitAnimationListener { it.remove() }
        enableEdgeToEdge()

        systemAuthenticator = SystemAuthenticator(this, viewModel)
        systemAuthenticator.prepare()
        systemAuthenticator.refreshEnrollment()

        viewModel.handleIntent(intent)
        viewModel.maintain()

        setContent {
            val state by viewModel.uiState.collectAsStateWithLifecycle()
            val pendingNavigation by viewModel.pendingNavigation.collectAsStateWithLifecycle()
            val systemAuthEnrollmentMissing by systemAuthenticator.enrollmentMissing.collectAsStateWithLifecycle()
            val connectionStatus by connectionStatusObserver.status.collectAsStateWithLifecycle()
            val connectionBannerState = remember { ConnectionBannerState() }
            LaunchedEffect(connectionStatus) {
                connectionBannerState.update(connectionStatus.bannerTitleRes()?.let(::getString))
            }
            val appearance by viewModel.appearance.collectAsStateWithLifecycle()
            val darkTheme = when (appearance) {
                Appearance.System -> isSystemInDarkTheme()
                Appearance.Light -> false
                Appearance.Dark -> true
            }
            LaunchedEffect(darkTheme) { applySystemBarsAppearance(darkTheme) }

            CompositionLocalProvider(LocalConnectionBannerState provides connectionBannerState) {
                MainContent(
                    state = state,
                    darkTheme = darkTheme,
                    pendingNavigation = pendingNavigation,
                    systemAuthEnrollmentMissing = systemAuthEnrollmentMissing,
                    activeWalletConnectRequest = activeWalletConnectRequest,
                    walletConnectEnabled = viewModel.isWalletConnectEnabled,
                    onSystemAuthRequired = systemAuthenticator::authenticate,
                    onPendingNavigationConsumed = viewModel::consumePendingNavigation,
                    onOpenSystemAuthSettings = systemAuthenticator::openSettings,
                    onWalletConnectPairingToastShown = viewModel::dismissWalletConnectPairingToast,
                    onScanErrorShown = viewModel::dismissScanError,
                    onWalletConnectError = viewModel::showWalletConnectError,
                    onErrorDismiss = viewModel::resetError,
                )
            }
            RootWarningHost(darkTheme = darkTheme, onCancel = ::finishAffinity)
        }
    }

    private fun applySystemBarsAppearance(darkTheme: Boolean) {
        WindowCompat.getInsetsController(window, window.decorView).apply {
            isAppearanceLightStatusBars = !darkTheme
            isAppearanceLightNavigationBars = !darkTheme
        }
    }

    override fun onResume() {
        super.onResume()
        systemAuthenticator.refreshEnrollment()
        viewModel.onActivityResumed()
    }

    override fun onPause() {
        super.onPause()
        viewModel.onActivityPaused()
    }

    override fun onDestroy() {
        systemAuthenticator.cancel()
        super.onDestroy()
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        viewModel.handleIntent(intent)
    }

    override fun requestAuth(auth: AuthRequest, onSuccess: () -> Unit) {
        systemAuthenticator.requestAuth(auth, onSuccess)
    }
}

private fun ConnectionStatus.bannerTitleRes(): Int? = when (this) {
    ConnectionStatus.Online -> null
    ConnectionStatus.NoInternet -> R.string.errors_no_internet_connection
    ConnectionStatus.NoService -> R.string.errors_no_service_connection
}
