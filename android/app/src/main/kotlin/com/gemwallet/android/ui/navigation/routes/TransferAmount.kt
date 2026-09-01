package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.transfer_amount.presents.AmountScreen
import uniffi.gemstone.GemConfirmInput
import com.gemwallet.android.ui.navigation.paramsArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable

@Serializable
data class AmountRoute(val params: String) : NavKey

fun EntryProviderScope<NavKey>.amount(
    onCancel: () -> Unit,
    onConfirm: (GemConfirmInput) -> Unit,
) {
    entry<AmountRoute>(
        metadata = { key -> routeArguments(paramsArgument(key.params)) },
    ) {
        AmountScreen(onCancel = onCancel, onConfirm = onConfirm)
    }
}
