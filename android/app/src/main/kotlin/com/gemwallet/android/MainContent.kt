package com.gemwallet.android

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.features.settings.presents.lock.LockScreen
import com.gemwallet.android.features.settings.viewmodels.lock.models.AuthState
import com.gemwallet.android.features.settings.viewmodels.lock.models.LockUIState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.WalletApp
import com.gemwallet.android.ui.theme.WalletTheme
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
    onSystemAuthRequired: () -> Unit,
    onPendingNavigationConsumed: () -> Unit,
    onOpenSystemAuthSettings: () -> Unit,
    onWalletConnectPairingToastShown: () -> Unit,
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

        if (walletConnectEnabled) {
            MessageToast(
                visible = state.isWalletConnectPairingToastVisible,
                message = R.string.wallet_connect_connection_title,
                onShown = onWalletConnectPairingToastShown,
            )
        }
        MessageToast(
            visible = isWalletUnlocked && pendingNavigation is PendingNavigation.Loading,
            message = R.string.common_loading,
            onShown = {},
        )
        MessageToast(
            message = state.navigationError,
            onShown = onErrorDismiss,
        )
        ErrorDialog(
            error = state.walletConnectError ?: unsupportedWalletConnectError,
            onDismiss = onErrorDismiss,
        )
    }
}
