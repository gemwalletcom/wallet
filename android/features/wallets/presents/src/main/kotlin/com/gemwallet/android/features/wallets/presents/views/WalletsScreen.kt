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
import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.gemwallet.android.features.wallet.presents.dialogs.ConfirmWalletDeleteDialog
import com.gemwallet.android.features.wallets.viewmodels.WalletsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemWalletPlaceholder
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSubtitle

@Composable
fun WalletsScreen(onCreateWallet: () -> Unit, onImportWallet: () -> Unit, onEditWallet: (WalletId) -> Unit, onSelectWallet: () -> Unit, onBoard: () -> Unit, onCancel: () -> Unit) {
    val viewModel: WalletsViewModel = hiltViewModel()
    val wallets by viewModel.wallets.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)
    val walletSections = remember(wallets) {
        wallets.toWalletSections()
    }

    var deleteWalletId by remember { mutableStateOf<WalletId?>(null) }

    WalletsScene(
        pinnedWallets = walletSections.pinnedWallets,
        unpinnedWallets = walletSections.unpinnedWallets,
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
            walletName = walletSections.allWallets.firstOrNull { it.row.id == pendingDeleteWalletId.id }?.row?.name ?: "",
            onConfirm = {
                deleteWalletId = null
                viewModel.deleteWallet(walletId = pendingDeleteWalletId, onBoard)
            },
        ) {
            deleteWalletId = null
        }
    }
}

internal data class WalletSections(val pinnedWallets: List<WalletDataAggregate>, val unpinnedWallets: List<WalletDataAggregate>) {
    val allWallets: List<WalletDataAggregate>
        get() = pinnedWallets + unpinnedWallets
}

internal fun List<WalletDataAggregate>.toWalletSections(): WalletSections {
    val (pinnedWallets, unpinnedWallets) = partition { it.row.isPinned }
    return WalletSections(
        pinnedWallets = pinnedWallets,
        unpinnedWallets = unpinnedWallets,
    )
}

@Preview
@Composable
fun PreviewWalletScreen() {
    MaterialTheme {
        Box {
            WalletsScene(
                unpinnedWallets = listOf(
                    object : WalletDataAggregate {
                        override val isCurrent: Boolean = true
                        override val row: GemWalletRow = GemWalletRow(
                            id = "1",
                            name = "Foo wallet #1",
                            subtitle = GemWalletSubtitle.Multicoin,
                            placeholder = GemWalletPlaceholder.Multicoin,
                            showsWatchBadge = false,
                            isPinned = false,
                            hasAvatar = false,
                            imageUrl = null,
                        )
                    },
                    object : WalletDataAggregate {
                        override val isCurrent: Boolean = false
                        override val row: GemWalletRow = GemWalletRow(
                            id = "2",
                            name = "Foo wallet #2",
                            subtitle = GemWalletSubtitle.Multicoin,
                            placeholder = GemWalletPlaceholder.Multicoin,
                            showsWatchBadge = false,
                            isPinned = false,
                            hasAvatar = false,
                            imageUrl = null,
                        )
                    },
                    object : WalletDataAggregate {
                        override val isCurrent: Boolean = false
                        override val row: GemWalletRow = GemWalletRow(
                            id = "3",
                            name = "Foo wallet #3",
                            subtitle = GemWalletSubtitle.Multicoin,
                            placeholder = GemWalletPlaceholder.Multicoin,
                            showsWatchBadge = false,
                            isPinned = false,
                            hasAvatar = false,
                            imageUrl = null,
                        )
                    },
                ),
                pinnedWallets = listOf(

                    object : WalletDataAggregate {
                        override val isCurrent: Boolean = true
                        override val row: GemWalletRow = GemWalletRow(
                            id = "4",
                            name = "Foo wallet #4",
                            subtitle = GemWalletSubtitle.Multicoin,
                            placeholder = GemWalletPlaceholder.Multicoin,
                            showsWatchBadge = false,
                            isPinned = false,
                            hasAvatar = false,
                            imageUrl = null,
                        )
                    },
                ),
                onAction = {},
            )
        }
    }
}
