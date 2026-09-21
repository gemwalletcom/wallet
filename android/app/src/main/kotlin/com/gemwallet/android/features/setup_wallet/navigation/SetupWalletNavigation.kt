package com.gemwallet.android.features.setup_wallet.navigation

import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.setup_wallet.viewmodels.SetupWalletViewModel
import com.gemwallet.android.features.setup_wallet.views.SetupWalletScreen
import com.wallet.core.primitives.WalletId
import kotlinx.serialization.Serializable

@Serializable
data class SetupWalletRoute(val walletId: WalletId) : NavKey

fun EntryProviderScope<NavKey>.setupWalletScreen(onComplete: () -> Unit, onSelectImage: (WalletId) -> Unit) {
    entry<SetupWalletRoute> { key ->
        val viewModel = hiltViewModel<SetupWalletViewModel, SetupWalletViewModel.Factory>(
            creationCallback = { factory -> factory.create(key.walletId) },
        )
        SetupWalletScreen(
            onComplete = onComplete,
            onSelectImage = { onSelectImage(key.walletId) },
            viewModel = viewModel,
        )
    }
}
