package com.gemwallet.android.features.wallets.presents.views

import androidx.compose.foundation.layout.Box
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.tooling.preview.Preview
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.gemwallet.android.features.wallet.presents.dialogs.ConfirmWalletDeleteDialog
import com.gemwallet.android.features.wallets.viewmodels.WalletsViewModel
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType

@Composable
fun WalletsScreen(
    onCreateWallet: () -> Unit,
    onImportWallet: () -> Unit,
    onEditWallet: (WalletId) -> Unit,
    onSelectWallet: () -> Unit,
    onBoard: () -> Unit,
    onCancel: () -> Unit,
) {
    val viewModel: WalletsViewModel = hiltViewModel()
    val wallets by viewModel.wallets.collectAsStateWithLifecycle()
    val walletSections = remember(wallets) {
        wallets.toWalletSections()
    }

    var deleteWalletId by remember { mutableStateOf<WalletId?>(null) }

    WalletsScene(
        pinnedWallets = walletSections.pinnedWallets,
        unpinnedWallets = walletSections.unpinnedWallets,
        onCreate = onCreateWallet,
        onImport = onImportWallet,
        onEdit = onEditWallet,
        onSelectWallet = {
            viewModel.selectWallet(it)
            onSelectWallet()
        },
        onDeleteWallet = {
            deleteWalletId = it
        },
        onTogglePin = viewModel::togglePin,
        onCancel = onCancel,
    )

    deleteWalletId?.let { pendingDeleteWalletId ->
        ConfirmWalletDeleteDialog(
            walletName = walletSections.allWallets.firstOrNull { it.id == pendingDeleteWalletId.id }?.name ?: "",
            onConfirm = {
                deleteWalletId = null
                viewModel.deleteWallet(walletId = pendingDeleteWalletId, onBoard)
            }
        ) {
            deleteWalletId = null
        }
    }
}

internal data class WalletSections(
    val pinnedWallets: List<WalletDataAggregate>,
    val unpinnedWallets: List<WalletDataAggregate>,
) {
    val allWallets: List<WalletDataAggregate>
        get() = pinnedWallets + unpinnedWallets
}

internal fun List<WalletDataAggregate>.toWalletSections(): WalletSections {
    val (pinnedWallets, unpinnedWallets) = partition { it.isPinned }
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
                        override val id: String = "1"
                        override val name: String = "Foo wallet #1"
                        override val type: WalletType = WalletType.View
                        override val isCurrent: Boolean = true
                        override val isPinned: Boolean = false
                        override val walletChain: Chain = Chain.Ethereum
                        override val walletAddress: String = "0xsdlkgjdlkfglkdjfg"
                        override val imageUrl: String? = null
                    },
                    object : WalletDataAggregate {
                        override val id: String = "1"
                        override val name: String = "Foo wallet #3"
                        override val type: WalletType = WalletType.Multicoin
                        override val isCurrent: Boolean = false
                        override val isPinned: Boolean = false
                        override val walletChain: Chain = Chain.Ethereum
                        override val walletAddress: String = "0xsdlkgjdlkfglkdjfg"
                        override val imageUrl: String? = null
                    },
                    object : WalletDataAggregate {
                        override val id: String = "1"
                        override val name: String = "Foo wallet #2"
                        override val type: WalletType = WalletType.PrivateKey
                        override val isCurrent: Boolean = false
                        override val isPinned: Boolean = false
                        override val walletChain: Chain = Chain.Bitcoin
                        override val walletAddress: String = "0xsdlkgjdlkfglkdjfg"
                        override val imageUrl: String? = null
                    },
                ),
                pinnedWallets = listOf(

                    object : WalletDataAggregate {
                        override val id: String = "1"
                        override val name: String = "Foo wallet #4"
                        override val type: WalletType = WalletType.Multicoin
                        override val isCurrent: Boolean = true
                        override val isPinned: Boolean = true
                        override val walletChain: Chain = Chain.Bitcoin
                        override val walletAddress: String = "0xsdlkgjdlkfglkdjfg"
                        override val imageUrl: String? = null
                    },
                ),
                onEdit = {},
                onCreate = {},
                onImport = {},
                onSelectWallet = {},
                onDeleteWallet = {},
                onTogglePin = {},
                onCancel = {},
            )
        }
    }
}
