package com.gemwallet.android

import android.Manifest
import android.app.ActivityManager
import android.content.Intent
import android.graphics.PixelFormat
import android.os.Build
import android.os.Bundle
import android.view.View
import android.view.ViewTreeObserver
import android.view.WindowManager
import android.view.inspector.WindowInspector
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.ComposeView
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.core.view.WindowCompat
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import androidx.lifecycle.setViewTreeLifecycleOwner
import androidx.savedstate.setViewTreeSavedStateRegistryOwner
import com.gemwallet.android.application.notifications.NotificationPermissionRequests
import com.gemwallet.android.application.security.cases.AuthRequester
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.features.settings.presents.lock.LockScene
import com.gemwallet.android.features.settings.viewmodels.lock.LockViewModel
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.AppViewModel
import com.gemwallet.android.ui.LocalChainService
import com.gemwallet.android.ui.LocalDeeplinkService
import com.gemwallet.android.ui.LocalNavigationService
import com.gemwallet.android.ui.LocalStreamConnected
import com.gemwallet.android.ui.components.ConnectionBannerState
import com.gemwallet.android.ui.components.LocalConnectionBannerState
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.walletSurfaceColor
import com.wallet.core.primitives.Appearance
import com.wallet.core.primitives.ConnectionComponent
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemConnectionService
import uniffi.gemstone.GemDeeplinkService
import uniffi.gemstone.GemNavigationService
import java.util.Collections
import java.util.WeakHashMap
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity :
    FragmentActivity(),
    AuthRequester {
    private val viewModel: MainViewModel by viewModels()
    private val appViewModel: AppViewModel by viewModels()
    private val lockViewModel: LockViewModel by viewModels()
    private lateinit var systemAuthenticator: SystemAuthenticator

    @Inject lateinit var connectionStatusObserver: ConnectionStatusObserver

    @Inject lateinit var connectionService: GemConnectionService

    @Inject lateinit var notificationPermissionRequests: NotificationPermissionRequests

    @Inject lateinit var activeWalletConnectRequest: ActiveWalletConnectRequest

    @Inject lateinit var deeplinkService: GemDeeplinkService

    @Inject lateinit var navigationService: GemNavigationService

    @Inject lateinit var chainService: GemChainService

    private val privacyCover by lazy { PrivacyCover(lifecycleScope, ::isGemFocused) }
    private val watchedWindows = Collections.newSetFromMap(WeakHashMap<View, Boolean>())
    private val windowFocusListener = ViewTreeObserver.OnWindowFocusChangeListener { privacyCover.onFocusChanged(isGemFocused()) }
    private var coverView: ComposeView? = null
    private var isDarkTheme = false

    private val notificationPermission = registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
        privacyCover.onPromptEnded()
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

        systemAuthenticator = SystemAuthenticator(this, lockViewModel, privacyCover)
        systemAuthenticator.prepare()
        systemAuthenticator.refreshEnrollment()

        if (savedInstanceState == null) {
            viewModel.pendIntent(intent)
        }
        viewModel.maintain(isUnlocked = lockViewModel.uiState.map { it.isUnlocked })

        lifecycleScope.launch {
            privacyCover.isCovered.collect { isCovered ->
                if (isCovered) {
                    showCover()
                    lockViewModel.onLeave()
                } else {
                    hideCover()
                    lockViewModel.onReturn()
                }
            }
        }

        lifecycleScope.launch {
            lockViewModel.observeAuthRequired().collect { isLockEnabled ->
                privacyCover.isLockEnabled = isLockEnabled
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                    setRecentsScreenshotEnabled(!isLockEnabled)
                }
            }
        }

        lifecycleScope.launch {
            repeatOnLifecycle(Lifecycle.State.STARTED) {
                notificationPermissionRequests.requests.collect { request ->
                    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
                        request.complete(false)
                        return@collect
                    }
                    pendingNotificationPermission = request
                    privacyCover.onPromptShown()
                    notificationPermission.launch(Manifest.permission.POST_NOTIFICATIONS)
                }
            }
        }

        setContent {
            val state by viewModel.uiState.collectAsStateWithLifecycle()
            val lockState by lockViewModel.uiState.collectAsStateWithLifecycle()
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
                if (!GemConstants.connectionBannerEnabled) return@LaunchedEffect
                delay(GemConstants.connectionBannerSettleDelay)
                connectionBannerState.update(connectionStatus.stringRes()?.let(::getString))
            }
            val appearance by viewModel.appearance.collectAsStateWithLifecycle()
            val darkTheme = when (appearance) {
                Appearance.System -> isSystemInDarkTheme()
                Appearance.Light -> false
                Appearance.Dark -> true
            }
            LaunchedEffect(darkTheme) {
                isDarkTheme = darkTheme
                setSystemBarsAppearance(darkTheme)
                setRecentsAppearance(darkTheme)
            }

            CompositionLocalProvider(
                LocalConnectionBannerState provides connectionBannerState,
                LocalStreamConnected provides streamConnected,
                LocalDeeplinkService provides deeplinkService,
                LocalNavigationService provides navigationService,
                LocalChainService provides chainService,
            ) {
                MainContent(
                    state = state,
                    lockState = lockState,
                    darkTheme = darkTheme,
                    pendingNavigation = pendingNavigation,
                    systemAuthEnrollmentMissing = systemAuthEnrollmentMissing,
                    activeWalletConnectRequest = activeWalletConnectRequest,
                    walletConnectEnabled = viewModel.isWalletConnectEnabled,
                    toastEvents = viewModel.toastEvents,
                    onSystemAuthRequired = systemAuthenticator::authenticate,
                    onPendingNavigationConsumed = viewModel::consumePendingNavigation,
                    onOpenSystemAuthSettings = systemAuthenticator::openSettings,
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

    private fun setRecentsAppearance(darkTheme: Boolean) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            val color = walletSurfaceColor(darkTheme).toArgb()
            setTaskDescription(ActivityManager.TaskDescription.Builder().setBackgroundColor(color).setStatusBarColor(color).setNavigationBarColor(color).build())
        }
    }

    private fun isGemFocused(): Boolean {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) return true
        val windows = WindowInspector.getGlobalWindowViews().filter { it !== coverView }
        windows.filter(watchedWindows::add).forEach { it.viewTreeObserver.addOnWindowFocusChangeListener(windowFocusListener) }
        return windows.any { it.hasWindowFocus() }
    }

    private fun showCover() {
        if (coverView != null || isFinishing) return
        val cover = ComposeView(this).apply {
            setViewTreeLifecycleOwner(this@MainActivity)
            setViewTreeSavedStateRegistryOwner(this@MainActivity)
            setContent { WalletTheme(darkTheme = isDarkTheme) { LockScene(logo = R.drawable.ic_splash_screen) } }
        }
        val params = WindowManager.LayoutParams(
            WindowManager.LayoutParams.TYPE_APPLICATION,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN or WindowManager.LayoutParams.FLAG_LAYOUT_NO_LIMITS,
            PixelFormat.OPAQUE,
        ).apply { layoutInDisplayCutoutMode = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES }
        windowManager.addView(cover, params)
        coverView = cover
    }

    private fun hideCover() {
        coverView?.let(windowManager::removeView)
        coverView = null
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        privacyCover.onFocusChanged(isGemFocused())
    }

    override fun onStop() {
        super.onStop()
        privacyCover.onStop()
    }

    override fun onResume() {
        super.onResume()
        systemAuthenticator.refreshEnrollment()
        lockViewModel.onReturn()
    }

    override fun onPause() {
        super.onPause()
        lockViewModel.onLeave()
    }

    override fun onDestroy() {
        hideCover()
        if (!isChangingConfigurations) {
            systemAuthenticator.cancel()
        }
        super.onDestroy()
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        viewModel.pendIntent(intent)
    }

    override fun requestAuth(auth: AuthRequest, onCancel: () -> Unit, onSuccess: () -> Unit) {
        systemAuthenticator.requestAuth(auth, onCancel, onSuccess)
    }
}
