package com.gemwallet.android.features.wallets.presents.views

import androidx.compose.foundation.layout.Box
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.wallet.presents.dialogs.ConfirmWalletDeleteDialog
import com.gemwallet.android.features.wallets.viewmodels.WalletsViewModel
import com.gemwallet.android.features.wallets.viewmodels.models.WalletItemUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.WalletRowUIModel
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.wallet.core.primitives.WalletId

@Composable
fun WalletsScreen(onCreateWallet: () -> Unit, onImportWallet: () -> Unit, onEditWallet: (WalletId) -> Unit, onSelectWallet: () -> Unit, onBoard: () -> Unit, onCancel: () -> Unit) {
    val viewModel: WalletsViewModel = hiltViewModel()
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    var deleteWalletId by remember { mutableStateOf<WalletId?>(null) }

    WalletsScene(
        pinnedWallets = uiState.pinned,
        unpinnedWallets = uiState.unpinned,
        snackbar = snackbar,
        onAction = { action ->
            when (action) {
                WalletsAction.Create -> onCreateWallet()
                WalletsAction.Import -> onImportWallet()
                is WalletsAction.Edit -> onEditWallet(action.walletId)
                is WalletsAction.Select -> viewModel.selectWallet(action.walletId, onSelectWallet)
                is WalletsAction.Delete -> deleteWalletId = action.walletId
                is WalletsAction.TogglePin -> viewModel.togglePin(action.walletId)
                WalletsAction.Cancel -> onCancel()
            }
        },
    )

    deleteWalletId?.let { pendingDeleteWalletId ->
        ConfirmWalletDeleteDialog(
            walletName = uiState.name(pendingDeleteWalletId),
            onConfirm = {
                deleteWalletId = null
                viewModel.deleteWallet(walletId = pendingDeleteWalletId, onBoard)
            },
        ) {
            deleteWalletId = null
        }
    }
}

@Preview
@Composable
fun PreviewWalletScreen() {
    val wallet = { id: String, name: String, isCurrent: Boolean, isPinned: Boolean ->
        WalletItemUIModel(
            walletId = WalletId(id),
            row = WalletRowUIModel(id = id, name = name, subtitle = "Multicoin", icon = R.drawable.multicoin_wallet, supportIcon = null),
            isCurrent = isCurrent,
            isPinned = isPinned,
        )
    }
    MaterialTheme {
        Box {
            WalletsScene(
                unpinnedWallets = listOf(
                    wallet("1", "Foo wallet #1", true, false),
                    wallet("2", "Foo wallet #2", false, false),
                    wallet("3", "Foo wallet #3", false, false),
                ),
                pinnedWallets = listOf(wallet("4", "Foo wallet #4", true, true)),
                onAction = {},
            )
        }
    }
}
