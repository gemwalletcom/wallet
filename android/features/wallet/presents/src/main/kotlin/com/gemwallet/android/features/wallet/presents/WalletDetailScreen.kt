package com.gemwallet.android.features.wallet.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.wallet.PrivateKeyExportInput
import com.gemwallet.android.domains.wallet.WalletSecretInput
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.features.wallet.viewmodels.WalletDetailViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemSecretExport
import uniffi.gemstone.GemWalletDeletion
import uniffi.gemstone.GemWalletSecretKind

@Composable
fun WalletDetailScreen(
    onPhraseShow: (WalletSecretInput) -> Unit,
    onPrivateKeyShow: (PrivateKeyExportInput) -> Unit,
    onSelectImage: (WalletId) -> Unit,
    onBoard: () -> Unit,
    onCancel: () -> Unit,
    viewModel: WalletDetailViewModel = hiltViewModel(),
) {
    val wallet by viewModel.details.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)
    val deletion by viewModel.deletion.collectAsStateWithLifecycle()

    LaunchedEffect(deletion) {
        val result = deletion ?: return@LaunchedEffect
        viewModel.onDeletionHandled()
        when (result) {
            GemWalletDeletion.WALLETS_REMAINING -> onCancel()
            GemWalletDeletion.LAST_WALLET_DELETED -> onBoard()
        }
    }

    WalletDetailScene(
        wallet = wallet,
        snackbar = snackbar,
        onAction = { action ->
            when (action) {
                is WalletDetailAction.SetName -> viewModel.setWalletName(action.name)

                WalletDetailAction.SelectImage -> wallet?.row?.id?.let { onSelectImage(WalletId(it)) }

                is WalletDetailAction.ShowSecret -> wallet?.row?.id?.let { id ->
                    val walletId = WalletId(id)
                    when (val export = action.export) {
                        is GemSecretExport.Secret -> onPhraseShow(WalletSecretInput(walletId, export.kind))
                        is GemSecretExport.PrivateKey -> onPrivateKeyShow(PrivateKeyExportInput(walletId, export.chain.requireChain()))
                        GemSecretExport.PrivateKeyAccounts -> onPrivateKeyShow(PrivateKeyExportInput(walletId, null))
                    }
                }

                WalletDetailAction.Delete -> viewModel.delete()

                WalletDetailAction.Cancel -> onCancel()
            }
        },
    )
}
