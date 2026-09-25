package com.gemwallet.android.features.wallets.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.wallets.viewmodels.WalletImageViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState

@Composable
fun WalletImageNavScreen(onCancel: () -> Unit, viewModel: WalletImageViewModel = hiltViewModel()) {
    val wallet by viewModel.details.collectAsStateWithLifecycle()
    val nftImages by viewModel.nftImages.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    WalletImageScene(
        wallet = wallet,
        emojis = viewModel.emojis,
        nftImages = nftImages,
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
