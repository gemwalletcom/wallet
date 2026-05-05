package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.features.transfer_amount.presents.AmountPerpetualNavScreen
import com.gemwallet.android.features.transfer_amount.presents.AmountScreen
import com.gemwallet.android.ui.navigation.paramsArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable

@Serializable
data class AmountRoute(val params: String) : NavKey

@Serializable
data class PerpetualAmountRoute(val params: String) : NavKey

fun EntryProviderScope<NavKey>.amount(
    onCancel: () -> Unit,
    onConfirm: (ConfirmParams) -> Unit,
) {
    entry<AmountRoute>(
        metadata = { key -> routeArguments(paramsArgument(key.params)) },
    ) {
        AmountScreen(onCancel = onCancel, onConfirm = onConfirm)
    }

    entry<PerpetualAmountRoute>(
        metadata = { key -> routeArguments(paramsArgument(key.params)) },
    ) {
        AmountPerpetualNavScreen(
            onConfirm = onConfirm,
            onClose = onCancel
        )
    }
}
