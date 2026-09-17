package com.gemwallet.android.features.wallet.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.wallet.WalletSecretInput
import com.gemwallet.android.features.wallet.viewmodels.WalletViewModel
import com.wallet.core.primitives.WalletId

@Composable
fun WalletNavScreen(
    onPhraseShow: (WalletSecretInput) -> Unit,
    onSelectImage: (WalletId) -> Unit,
    onBoard: () -> Unit,
    onCancel: () -> Unit,
    viewModel: WalletViewModel = hiltViewModel(),
) {
    val wallet by viewModel.wallet.collectAsStateWithLifecycle()
    val secret by viewModel.secret.collectAsStateWithLifecycle()

    WalletScene(
        wallet = wallet,
        secret = secret,
        onAction = { action ->
            when (action) {
                is WalletAction.SetName -> viewModel.setWalletName(action.name)
                WalletAction.SelectImage -> wallet?.id?.let(onSelectImage)
                is WalletAction.ShowPhrase -> onPhraseShow(action.input)
                WalletAction.Delete -> viewModel.delete(onBoard, onCancel)
                WalletAction.Cancel -> onCancel()
            }
        },
    )
}
