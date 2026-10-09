package com.gemwallet.android

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.features.settings.presents.lock.LockScreen
import com.gemwallet.android.features.settings.viewmodels.lock.models.AuthState
import com.gemwallet.android.features.settings.viewmodels.lock.models.LockUIState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.WalletApp
import com.gemwallet.android.ui.components.screen.SnackbarHost
import com.gemwallet.android.ui.components.screen.ToastEffect
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.mainActionHeight
import com.gemwallet.android.ui.theme.paddingLarge
import kotlinx.coroutines.flow.Flow
import com.gemwallet.android.R as AppR

@Composable
internal fun MainContent(
    state: MainViewModel.MainUIState,
    lockState: LockUIState,
    darkTheme: Boolean,
    pendingNavigation: PendingNavigation?,
    systemAuthEnrollmentMissing: Boolean,
    activeWalletConnectRequest: ActiveWalletConnectRequest,
    walletConnectEnabled: Boolean,
    toastEvents: Flow<ToastMessage>,
    onSystemAuthRequired: () -> Unit,
    onPendingNavigationConsumed: () -> Unit,
    onOpenSystemAuthSettings: () -> Unit,
    onWalletConnectError: (String) -> Unit,
    onErrorDismiss: () -> Unit,
) {
    val pendingDestination = pendingNavigation as? PendingNavigation.Routes
    val pendingRoutes = pendingDestination?.routes.orEmpty()
    val canAttemptSystemAuth = !systemAuthEnrollmentMissing
    val requiresAuthPrompt = lockState.initialAuth == AuthState.Required || lockState.authState == AuthState.Required
    val isWalletUnlocked = lockState.isUnlocked
    val isEnrollmentRequired = lockState.initialAuth == AuthState.Required && systemAuthEnrollmentMissing
    val unlockedPendingRoutes = if (isWalletUnlocked) pendingRoutes else emptyList()
    val unsupportedWalletConnectError = if (state.isWalletConnectUnsupportedVisible) {
        "${stringResource(R.string.wallet_connect_title)}: ${stringResource(R.string.errors_not_supported)} (${BuildConfig.FLAVOR})"
    } else {
        null
    }
    var isWalletContentReady by remember { mutableStateOf(lockState.hasUnlockedApp) }
    val onWalletContentReady: () -> Unit = remember { { isWalletContentReady = true } }

    LaunchedEffect(requiresAuthPrompt, canAttemptSystemAuth, lockState.authPromptRequest) {
        if (requiresAuthPrompt && canAttemptSystemAuth) {
            onSystemAuthRequired()
        }
    }

    WalletTheme(darkTheme = darkTheme) {
        Box(modifier = Modifier.fillMaxSize()) {
            if (lockState.hasUnlockedApp) {
                WalletApp(
                    pendingRoutes = unlockedPendingRoutes,
                    pendingTab = pendingDestination?.tab,
                    onPendingNavigationConsumed = onPendingNavigationConsumed,
                    onContentReady = onWalletContentReady,
                    activeWalletConnectRequest = activeWalletConnectRequest.takeIf { walletConnectEnabled },
                    onWalletConnectError = onWalletConnectError,
                )
                AppToastHost(toastEvents)
            }

            if (isEnrollmentRequired) {
                SystemAuthEnrollmentRequired(
                    onOpenSettings = onOpenSystemAuthSettings,
                )
            } else {
                LockScreen(
                    isContentReady = isWalletContentReady,
                    logo = AppR.drawable.ic_splash_screen,
                )
            }
        }

        ErrorDialog(
            error = state.walletConnectError ?: unsupportedWalletConnectError ?: state.startupError,
            onDismiss = onErrorDismiss,
        )
    }
}

@Composable
private fun AppToastHost(events: Flow<ToastMessage>) {
    val snackbar = remember { SnackbarHostState() }
    ToastEffect(events, snackbar)
    Box(
        modifier = Modifier
            .fillMaxSize()
            .windowInsetsPadding(WindowInsets.navigationBars.union(WindowInsets.ime))
            .padding(bottom = mainActionHeight + paddingLarge),
        contentAlignment = Alignment.BottomCenter,
    ) {
        SnackbarHost(snackbar)
    }
}
