package com.gemwallet.android.features.wallet.presents

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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.wallet.presents.dialogs.ConfirmWalletDeleteDialog
import com.gemwallet.android.features.wallet.viewmodels.WalletsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.localization.string
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemWalletPlaceholder
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSection
import uniffi.gemstone.GemWalletSectionKind
import uniffi.gemstone.GemWalletSubtitle

@Composable
fun WalletsScreen(onCreateWallet: () -> Unit, onImportWallet: () -> Unit, onEditWallet: (WalletId) -> Unit, onSelectWallet: () -> Unit, onBoard: () -> Unit, onCancel: () -> Unit) {
    val viewModel: WalletsViewModel = hiltViewModel()
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    var deleteRow by remember { mutableStateOf<GemWalletRow?>(null) }

    WalletsScene(
        sections = sections,
        snackbar = snackbar,
        onAction = { action ->
            when (action) {
                WalletsAction.Create -> onCreateWallet()
                WalletsAction.Import -> onImportWallet()
                is WalletsAction.Edit -> onEditWallet(WalletId(action.row.id))
                is WalletsAction.Select -> viewModel.selectWallet(action.row, onSelectWallet)
                is WalletsAction.Delete -> deleteRow = action.row
                is WalletsAction.TogglePin -> viewModel.togglePin(action.row)
                WalletsAction.Cancel -> onCancel()
            }
        },
    )

    deleteRow?.let { row ->
        ConfirmWalletDeleteDialog(
            prompt = row.deletePrompt.string(LocalContext.current),
            onConfirm = {
                deleteRow = null
                viewModel.deleteWallet(row, onBoard)
            },
        ) {
            deleteRow = null
        }
    }
}

@Preview
@Composable
fun PreviewWalletsScene() {
    val wallet = { id: String, name: String, isPinned: Boolean, isCurrent: Boolean ->
        GemWalletRow(
            id = id,
            name = name,
            subtitle = GemWalletSubtitle.Multicoin,
            placeholder = GemWalletPlaceholder.Multicoin,
            showsWatchBadge = false,
            isPinned = isPinned,
            isCurrent = isCurrent,
            hasAvatar = false,
            imageUrl = null,
            deletePrompt = GemLocalizedText.DeleteConfirmation(name),
        )
    }
    MaterialTheme {
        Box {
            WalletsScene(
                sections = listOf(
                    GemWalletSection(GemWalletSectionKind.PINNED, listOf(wallet("4", "Foo wallet #4", true, false))),
                    GemWalletSection(
                        GemWalletSectionKind.WALLETS,
                        listOf(
                            wallet("1", "Foo wallet #1", false, true),
                            wallet("2", "Foo wallet #2", false, false),
                            wallet("3", "Foo wallet #3", false, false),
                        ),
                    ),
                ),
                onAction = {},
            )
        }
    }
}
