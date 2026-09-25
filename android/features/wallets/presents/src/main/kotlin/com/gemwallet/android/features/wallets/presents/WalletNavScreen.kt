package com.gemwallet.android.features.wallets.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.wallet.WalletSecretInput
import com.gemwallet.android.features.wallets.viewmodels.WalletViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.wallet.core.primitives.WalletId

@Composable
fun WalletNavScreen(onPhraseShow: (WalletSecretInput) -> Unit, onSelectImage: (WalletId) -> Unit, onBoard: () -> Unit, onCancel: () -> Unit, viewModel: WalletViewModel = hiltViewModel()) {
    val wallet by viewModel.details.collectAsStateWithLifecycle()
    val secret by viewModel.secret.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    WalletScene(
        wallet = wallet,
        secret = secret,
        snackbar = snackbar,
        onAction = { action ->
            when (action) {
                is WalletAction.SetName -> viewModel.setWalletName(action.name)
                WalletAction.SelectImage -> wallet?.walletId?.let(onSelectImage)
                is WalletAction.ShowPhrase -> onPhraseShow(action.input)
                WalletAction.Delete -> viewModel.delete(onBoard, onCancel)
                WalletAction.Cancel -> onCancel()
            }
        },
    )
}
