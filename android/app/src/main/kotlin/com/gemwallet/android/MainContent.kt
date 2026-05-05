package com.gemwallet.android

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import com.gemwallet.android.model.AuthState
import com.gemwallet.android.ui.WalletApp
import com.gemwallet.android.ui.theme.WalletTheme

@Composable
internal fun MainContent(
    state: MainViewModel.MainUIState,
    pendingNavigation: PendingNavigation?,
    systemAuthEnrollmentMissing: Boolean,
    walletConnectViewModel: WalletConnectViewModel,
    onSystemAuthRequired: () -> Unit,
    onPreparePendingNavigation: () -> Unit,
    onIntentConsumed: () -> Unit,
    onOpenSystemAuthSettings: () -> Unit,
    onWalletConnectPairingToastShown: () -> Unit,
    onWalletConnectErrorDismiss: () -> Unit,
) {
    val pendingRoute = (pendingNavigation as? PendingNavigation.Route)?.route
    val canAttemptSystemAuth = !systemAuthEnrollmentMissing
    val requiresAuthPrompt = state.initialAuth == AuthState.Required || state.authState == AuthState.Required
    val isWalletUnlocked = state.initialAuth == AuthState.Success
    val isEnrollmentRequired = state.initialAuth == AuthState.Required && systemAuthEnrollmentMissing
    val walletConnectOverlay = rememberWalletConnectOverlay(walletConnectViewModel)

    LaunchedEffect(requiresAuthPrompt, canAttemptSystemAuth, state.authPromptRequest) {
        if (requiresAuthPrompt && canAttemptSystemAuth) {
            onSystemAuthRequired()
        }
    }

    WalletTheme {
        LaunchedEffect(pendingNavigation, isWalletUnlocked) {
            if (pendingNavigation is PendingNavigation.RawIntent && isWalletUnlocked) {
                onPreparePendingNavigation()
            }
        }

        when {
            isWalletUnlocked -> WalletApp(
                pendingRoute = pendingRoute,
                onIntentConsumed = onIntentConsumed,
                walletConnectOverlay = walletConnectOverlay,
            )
            isEnrollmentRequired -> SystemAuthEnrollmentRequired(
                onOpenSettings = onOpenSystemAuthSettings,
            )
            else -> LockedSplash()
        }

        WalletConnectPairingToast(
            visible = state.isWalletConnectPairingToastVisible,
            onShown = onWalletConnectPairingToastShown,
        )
        WalletConnectErrorDialog(
            error = state.walletConnectError,
            onDismiss = onWalletConnectErrorDismiss,
        )
    }
}
