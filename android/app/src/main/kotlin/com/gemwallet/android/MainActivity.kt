package com.gemwallet.android

import android.Manifest
import android.content.Intent
import android.os.Build
import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.core.view.WindowCompat
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import com.gemwallet.android.application.notifications.NotificationPermissionRequests
import com.gemwallet.android.application.security.cases.AuthRequester
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.localization.stringRes
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.AppViewModel
import com.gemwallet.android.ui.LocalAddressService
import com.gemwallet.android.ui.LocalAssetsService
import com.gemwallet.android.ui.LocalChainService
import com.gemwallet.android.ui.LocalConnectionStatus
import com.gemwallet.android.ui.LocalDeeplinkService
import com.gemwallet.android.ui.LocalNavigationService
import com.gemwallet.android.ui.LocalStreamConnected
import com.gemwallet.android.ui.components.ConnectionBannerState
import com.gemwallet.android.ui.components.LocalConnectionBannerState
import com.wallet.core.primitives.Appearance
import com.wallet.core.primitives.ConnectionComponent
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAddressService
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemDeeplinkService
import uniffi.gemstone.GemNavigationService
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity :
    FragmentActivity(),
    AuthRequester {
    private val viewModel: MainViewModel by viewModels()
    private val appViewModel: AppViewModel by viewModels()
    private lateinit var systemAuthenticator: SystemAuthenticator

    @Inject lateinit var connectionStatusObserver: ConnectionStatusObserver

    @Inject lateinit var notificationPermissionRequests: NotificationPermissionRequests

    @Inject lateinit var activeWalletConnectRequest: ActiveWalletConnectRequest

    @Inject lateinit var addressService: GemAddressService

    @Inject lateinit var deeplinkService: GemDeeplinkService

    @Inject lateinit var navigationService: GemNavigationService

    @Inject lateinit var assetsService: GemAssetsService

    @Inject lateinit var chainService: GemChainService

    private val notificationPermission = registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
        pendingNotificationPermission?.complete(granted)
        pendingNotificationPermission = null
    }
    private var pendingNotificationPermission: CompletableDeferred<Boolean>? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        val splashScreen = installSplashScreen()
        super.onCreate(savedInstanceState)
        splashScreen.setKeepOnScreenCondition { !appViewModel.launchReadyState.value }
        splashScreen.setOnExitAnimationListener { it.remove() }
        enableEdgeToEdge()

        systemAuthenticator = SystemAuthenticator(this, viewModel)
        systemAuthenticator.prepare()
        systemAuthenticator.refreshEnrollment()

        viewModel.pendIntent(intent)
        viewModel.maintain()

        lifecycleScope.launch {
            repeatOnLifecycle(Lifecycle.State.STARTED) {
                notificationPermissionRequests.requests.collect { request ->
                    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
                        request.complete(false)
                        return@collect
                    }
                    pendingNotificationPermission = request
                    notificationPermission.launch(Manifest.permission.POST_NOTIFICATIONS)
                }
            }
        }

        setContent {
            val state by viewModel.uiState.collectAsStateWithLifecycle()
            val pendingNavigation by viewModel.pendingNavigation.collectAsStateWithLifecycle()
            val systemAuthEnrollmentMissing by systemAuthenticator.enrollmentMissing.collectAsStateWithLifecycle()
            val connectionStatus by connectionStatusObserver.status.collectAsStateWithLifecycle()
            val streamConnected = remember {
                connectionStatusObserver.isHealthyByComponent
                    .map { it[ConnectionComponent.Stream] == true }
                    .stateIn(lifecycleScope, SharingStarted.Eagerly, false)
            }
            val connectionBannerState = remember { ConnectionBannerState() }
            LaunchedEffect(connectionStatus) {
                connectionBannerState.update(connectionStatus.stringRes()?.let(::getString))
            }
            val appearance by viewModel.appearance.collectAsStateWithLifecycle()
            val darkTheme = when (appearance) {
                Appearance.System -> isSystemInDarkTheme()
                Appearance.Light -> false
                Appearance.Dark -> true
            }
            LaunchedEffect(darkTheme) { setSystemBarsAppearance(darkTheme) }

            CompositionLocalProvider(
                LocalConnectionBannerState provides connectionBannerState,
                LocalConnectionStatus provides connectionStatusObserver.status,
                LocalStreamConnected provides streamConnected,
                LocalAddressService provides addressService,
                LocalAssetsService provides assetsService,
                LocalDeeplinkService provides deeplinkService,
                LocalNavigationService provides navigationService,
                LocalChainService provides chainService,
            ) {
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

    private fun setSystemBarsAppearance(darkTheme: Boolean) {
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
        viewModel.pendIntent(intent)
    }

    override fun requestAuth(auth: AuthRequest, onSuccess: () -> Unit) {
        systemAuthenticator.requestAuth(auth, onSuccess)
    }
}
