package com.gemwallet.android.features.create_wallet.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.create_wallet.views.CreateWalletScreen
import com.gemwallet.android.features.create_wallet.views.PhraseAlertDialog
import com.wallet.core.primitives.WalletId
import kotlinx.serialization.Serializable

@Serializable
data object CreateWalletAlertRoute : NavKey

@Serializable
data object CreateWalletRoute : NavKey

fun EntryProviderScope<NavKey>.createWalletScreen(
    onCreateWallet: () -> Unit,
    onCancel: () -> Unit,
    onCreated: (walletId: WalletId?) -> Unit,
) {

    entry<CreateWalletAlertRoute> {
        PhraseAlertDialog(onAccept = onCreateWallet, onCancel = onCancel)
    }

    entry<CreateWalletRoute> {
        CreateWalletScreen(
            onCancel = onCancel,
            onCreated = onCreated,
        )
    }
}
