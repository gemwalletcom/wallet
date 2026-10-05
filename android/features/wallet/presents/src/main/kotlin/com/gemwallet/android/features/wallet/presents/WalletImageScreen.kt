package com.gemwallet.android.features.wallet.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.wallet.viewmodels.WalletImageViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState

@Composable
fun WalletImageScreen(onCancel: () -> Unit, viewModel: WalletImageViewModel = hiltViewModel()) {
    val wallet by viewModel.details.collectAsStateWithLifecycle()
    val avatars by viewModel.avatars.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    WalletImageScene(
        wallet = wallet,
        emojis = viewModel.emojis,
        avatars = avatars,
        snackbar = snackbar,
        onAction = { action ->
            when (action) {
                is WalletImageAction.SetEmoji -> viewModel.setEmoji(action.emoji, action.backgroundColor)
                is WalletImageAction.SetNftImage -> viewModel.setNftImage(action.url)
                WalletImageAction.ResetToDefault -> viewModel.resetToDefault()
                WalletImageAction.Close -> onCancel()
            }
        },
    )
}
