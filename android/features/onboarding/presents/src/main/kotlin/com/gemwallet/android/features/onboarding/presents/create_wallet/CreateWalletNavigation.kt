package com.gemwallet.android.features.onboarding.presents.create_wallet

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable

@Serializable
data object CreateWalletSecurityReminderRoute : NavKey

@Serializable
data object CreateWalletRoute : NavKey

fun EntryProviderScope<NavKey>.createWalletScreen(onCreateWallet: () -> Unit, onCancel: () -> Unit, onCreated: () -> Unit) {
    entry<CreateWalletSecurityReminderRoute> {
        SecurityReminderScene(onAccept = onCreateWallet, onCancel = onCancel)
    }

    entry<CreateWalletRoute> {
        CreateWalletScreen(
            onCancel = onCancel,
            onCreated = onCreated,
        )
    }
}
