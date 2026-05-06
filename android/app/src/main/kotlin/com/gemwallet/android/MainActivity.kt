package com.gemwallet.android

import android.content.Intent
import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.runtime.getValue
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.cases.security.AuthRequester
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.AppViewModel
import dagger.hilt.android.AndroidEntryPoint

@AndroidEntryPoint
class MainActivity : FragmentActivity(), AuthRequester {
    private val viewModel: MainViewModel by viewModels()
    private val appViewModel: AppViewModel by viewModels()
    private val walletConnectViewModel: WalletConnectViewModel by viewModels()
    private lateinit var systemAuthenticator: SystemAuthenticator

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

            MainContent(
                state = state,
                pendingNavigation = pendingNavigation,
                systemAuthEnrollmentMissing = systemAuthEnrollmentMissing,
                walletConnectViewModel = walletConnectViewModel,
                onSystemAuthRequired = systemAuthenticator::authenticate,
                onIntentConsumed = viewModel::consumePendingNavigation,
                onOpenSystemAuthSettings = systemAuthenticator::openSettings,
                onWalletConnectPairingToastShown = viewModel::dismissWalletConnectPairingToast,
                onWalletConnectErrorDismiss = viewModel::resetWalletConnectError,
            )
            RootWarningHost(onCancel = ::finishAffinity)
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
