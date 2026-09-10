package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.Composable
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable

@Serializable
data class WalletConnectRequestRoute(val key: String) : NavKey

fun EntryProviderScope<NavKey>.walletConnectRequest(content: @Composable (String) -> Unit) {
    entry<WalletConnectRequestRoute> { route ->
        content(route.key)
    }
}
