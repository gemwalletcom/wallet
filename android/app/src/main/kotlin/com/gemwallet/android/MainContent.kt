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
import com.gemwallet.android.data.repositories.bridge.ActiveWalletConnectRequest
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.model.AuthState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.WalletApp
import com.gemwallet.android.ui.theme.WalletTheme
import com.wallet.core.primitives.AssetId

@Composable
internal fun MainContent(
    state: MainViewModel.MainUIState,
    darkTheme: Boolean,
    pendingNavigation: PendingNavigation?,
    systemAuthEnrollmentMissing: Boolean,
    activeWalletConnectRequest: ActiveWalletConnectRequest,
    walletConnectEnabled: Boolean,
    onSystemAuthRequired: () -> Unit,
    onPendingNavigationConsumed: () -> Unit,
    onOpenSystemAuthSettings: () -> Unit,
    onWalletConnectPairingToastShown: () -> Unit,
    onScanErrorShown: () -> Unit,
    onWalletConnectError: (String) -> Unit,
    onWalletConnectErrorDismiss: () -> Unit,
) {
    val pendingRoutes = (pendingNavigation as? PendingNavigation.Routes)?.routes.orEmpty()
    val canAttemptSystemAuth = !systemAuthEnrollmentMissing
    val requiresAuthPrompt = state.initialAuth == AuthState.Required || state.authState == AuthState.Required
    val isWalletUnlocked = state.initialAuth == AuthState.Success
    val isEnrollmentRequired = state.initialAuth == AuthState.Required && systemAuthEnrollmentMissing
    val unlockedPendingRoutes = if (isWalletUnlocked) pendingRoutes else emptyList()
    val unsupportedWalletConnectError = if (state.isWalletConnectUnsupportedVisible) {
        "${stringResource(R.string.wallet_connect_title)}: ${stringResource(R.string.errors_not_supported)} (${BuildConfig.FLAVOR})"
    } else {
        null
    }
    val walletConnectOverlay: @Composable ((AcquireAssetAction, AssetId) -> Unit) -> Unit = if (walletConnectEnabled) {
        rememberWalletConnectOverlay(activeWalletConnectRequest, onWalletConnectError)
    } else {
        remember { { _ -> } }
    }
    var isWalletContentReady by remember { mutableStateOf(state.hasUnlockedApp) }
    val onWalletContentReady: () -> Unit = remember { { isWalletContentReady = true } }
    val shouldShowLockedSplash = !isWalletUnlocked || !isWalletContentReady

    LaunchedEffect(requiresAuthPrompt, canAttemptSystemAuth, state.authPromptRequest) {
        if (requiresAuthPrompt && canAttemptSystemAuth) {
            onSystemAuthRequired()
        }
    }

    WalletTheme(darkTheme = darkTheme) {
        Box(modifier = Modifier.fillMaxSize()) {
            if (state.hasUnlockedApp) {
                WalletApp(
                    pendingRoutes = unlockedPendingRoutes,
                    onPendingNavigationConsumed = onPendingNavigationConsumed,
                    onContentReady = onWalletContentReady,
                    walletConnectOverlay = walletConnectOverlay,
                )
            }

            when {
                isEnrollmentRequired -> SystemAuthEnrollmentRequired(
                    onOpenSettings = onOpenSystemAuthSettings,
                )
                shouldShowLockedSplash -> LockedSplash()
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
            visible = state.isScanErrorVisible,
            message = R.string.errors_not_supported,
            onShown = onScanErrorShown,
        )
        WalletConnectErrorDialog(
            error = state.walletConnectError ?: unsupportedWalletConnectError,
            onDismiss = onWalletConnectErrorDismiss,
        )
    }
}
