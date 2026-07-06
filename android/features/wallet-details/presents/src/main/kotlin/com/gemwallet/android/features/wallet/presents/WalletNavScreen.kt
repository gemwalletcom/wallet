package com.gemwallet.android.features.wallet.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.wallet.viewmodels.WalletViewModel
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType

@Composable
fun WalletNavScreen(
    onPhraseShow: (WalletId, WalletType) -> Unit,
    onSelectImage: (WalletId) -> Unit,
    onBoard: () -> Unit,
    onCancel: () -> Unit,
    viewModel: WalletViewModel = hiltViewModel(),
) {
    val wallet by viewModel.wallet.collectAsStateWithLifecycle()

    WalletScene(
        wallet = wallet,
        onAction = { action ->
            when (action) {
                is WalletAction.SetName -> viewModel.setWalletName(action.name)
                WalletAction.SelectImage -> wallet?.id?.let(onSelectImage)
                is WalletAction.ShowPhrase -> onPhraseShow(action.walletId, action.walletType)
                WalletAction.Delete -> viewModel.delete(onBoard, onCancel)
                WalletAction.Cancel -> onCancel()
            }
        },
    )
}
