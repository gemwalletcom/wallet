package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.assets.presents.address.AddressDetailsScreen
import com.wallet.core.primitives.ChainAddress
import kotlinx.serialization.Serializable

@Serializable
data class AddressDetailsRoute(val chainAddress: ChainAddress) : NavKey

fun EntryProviderScope<NavKey>.addressDetailsScreen(onCancel: () -> Unit) {
    entry<AddressDetailsRoute> { key ->
        AddressDetailsScreen(
            chainAddress = key.chainAddress,
            onCancel = onCancel,
        )
    }
}
