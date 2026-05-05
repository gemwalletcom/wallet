package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.wallets.presents.views.WalletsScreen
import com.wallet.core.primitives.WalletId
import kotlinx.serialization.Serializable

@Serializable
data object WalletsRoute : NavKey

fun EntryProviderScope<NavKey>.walletsScreen(
    onCancel: () -> Unit,
    onCreateWallet: () -> Unit,
    onImportWallet: () -> Unit,
    onEditWallet: (WalletId) -> Unit,
    onSelectWallet: () -> Unit,
    onBoard: () -> Unit,
) {
    entry<WalletsRoute> {
        WalletsScreen(
            onCreateWallet = onCreateWallet,
            onImportWallet = onImportWallet,
            onEditWallet = onEditWallet,
            onSelectWallet = onSelectWallet,
            onBoard = onBoard,
            onCancel = onCancel
        )
    }
}
