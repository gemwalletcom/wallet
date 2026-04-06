package com.gemwallet.android.features.wallet.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.wallet.viewmodels.WalletViewModel
import com.wallet.core.primitives.WalletType

@Composable
fun WalletNavScreen(
    onPhraseShow: (String, WalletType) -> Unit,
    onBoard: () -> Unit,
    onCancel: () -> Unit,
    viewModel: WalletViewModel = hiltViewModel(),
) {
    val wallet by viewModel.wallet.collectAsStateWithLifecycle()

    WalletScene(
        wallet = wallet,
        onWalletName = viewModel::setWalletName,
        onPhraseShow = onPhraseShow,
        onDelete = { viewModel.delete(onBoard, onCancel) },
        onCancel = onCancel,
    )
}
