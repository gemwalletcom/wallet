package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.wallet.presents.WalletsAction
import com.gemwallet.android.features.wallet.presents.WalletsScreen
import kotlinx.serialization.Serializable

@Serializable
data object WalletsRoute : NavKey

fun EntryProviderScope<NavKey>.walletsScreen(onAction: (WalletsAction.Navigation) -> Unit, onSelectWallet: () -> Unit, onBoard: () -> Unit) {
    entry<WalletsRoute> {
        WalletsScreen(
            onAction = onAction,
            onSelectWallet = onSelectWallet,
            onBoard = onBoard,
        )
    }
}
