package com.gemwallet.android.features.wallets.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.wallet.WalletSecretInput
import com.gemwallet.android.features.wallets.viewmodels.WalletDetailViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.wallet.core.primitives.WalletId

@Composable
fun WalletDetailScreen(onPhraseShow: (WalletSecretInput) -> Unit, onSelectImage: (WalletId) -> Unit, onBoard: () -> Unit, onCancel: () -> Unit, viewModel: WalletDetailViewModel = hiltViewModel()) {
    val wallet by viewModel.details.collectAsStateWithLifecycle()
    val secret by viewModel.secret.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    WalletDetailScene(
        wallet = wallet,
        secret = secret,
        snackbar = snackbar,
        onAction = { action ->
            when (action) {
                is WalletDetailAction.SetName -> viewModel.setWalletName(action.name)
                WalletDetailAction.SelectImage -> wallet?.walletId?.let(onSelectImage)
                is WalletDetailAction.ShowPhrase -> onPhraseShow(action.input)
                WalletDetailAction.Delete -> viewModel.delete(onBoard, onCancel)
                WalletDetailAction.Cancel -> onCancel()
            }
        },
    )
}
