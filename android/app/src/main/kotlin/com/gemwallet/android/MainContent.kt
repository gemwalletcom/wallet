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
    onIntentConsumed: () -> Unit,
    onOpenSystemAuthSettings: () -> Unit,
    onWalletConnectPairingToastShown: () -> Unit,
    onWalletConnectErrorDismiss: () -> Unit,
) {
    val pendingRoutes = (pendingNavigation as? PendingNavigation.Route)?.routes.orEmpty()
    val canAttemptSystemAuth = !systemAuthEnrollmentMissing
    val requiresAuthPrompt = state.initialAuth == AuthState.Required || state.authState == AuthState.Required
    val isWalletUnlocked = state.initialAuth == AuthState.Success
    val isEnrollmentRequired = state.initialAuth == AuthState.Required && systemAuthEnrollmentMissing
    val unlockedPendingRoutes = if (isWalletUnlocked) pendingRoutes else emptyList()
    val walletConnectOverlay = rememberWalletConnectOverlay(walletConnectViewModel)
    var isWalletContentReady by remember { mutableStateOf(state.hasUnlockedApp) }
    val onWalletContentReady: () -> Unit = remember { { isWalletContentReady = true } }
    val shouldShowLockedSplash = !isWalletUnlocked || !isWalletContentReady

    LaunchedEffect(requiresAuthPrompt, canAttemptSystemAuth, state.authPromptRequest) {
        if (requiresAuthPrompt && canAttemptSystemAuth) {
            onSystemAuthRequired()
        }
    }

    WalletTheme {
        Box(modifier = Modifier.fillMaxSize()) {
            if (state.hasUnlockedApp) {
                WalletApp(
                    pendingRoutes = unlockedPendingRoutes,
                    onIntentConsumed = onIntentConsumed,
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
