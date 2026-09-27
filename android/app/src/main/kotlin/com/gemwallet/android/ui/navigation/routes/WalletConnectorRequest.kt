package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.Composable
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable

@Serializable
data class WalletConnectorRequestRoute(val key: String) : NavKey

fun EntryProviderScope<NavKey>.walletConnectorRequest(content: @Composable (String) -> Unit) {
    entry<WalletConnectorRequestRoute> { route ->
        content(route.key)
    }
}
