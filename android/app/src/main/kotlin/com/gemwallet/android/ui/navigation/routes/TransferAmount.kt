package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.features.transfer.presents.amount.AmountScreen
import com.gemwallet.android.ui.navigation.paramsArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable

@Serializable
data class AmountRoute(val params: String) : NavKey

fun EntryProviderScope<NavKey>.amount(onCancel: () -> Unit, onConfirm: (ConfirmTransferInput) -> Unit, onBuy: (AssetId) -> Unit) {
    entry<AmountRoute>(
        metadata = { key -> routeArguments(paramsArgument(key.params)) },
    ) {
        AmountScreen(onCancel = onCancel, onConfirm = onConfirm, onBuy = onBuy)
    }
}
