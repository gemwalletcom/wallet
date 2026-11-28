package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.features.transfer.presents.amount.AmountScreen
import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Contextual
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemAmountRequest

@Serializable
data class AmountRoute(val assetId: AssetId, val request: @Contextual GemAmountRequest) : NavKey

fun EntryProviderScope<NavKey>.amount(onCancel: () -> Unit, onConfirm: (ConfirmTransferInput) -> Unit, onBuy: (AssetId) -> Unit) {
    entry<AmountRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId), RouteArgument.Params to key.request.packRoutePayload()) },
    ) {
        AmountScreen(onCancel = onCancel, onConfirm = onConfirm, onBuy = onBuy)
    }
}
