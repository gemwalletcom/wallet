package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.remember
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.features.confirm.presents.ConfirmScreen
import com.gemwallet.android.ui.navigation.paramsArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable

@Serializable
data class ConfirmRoute(val params: String) : NavKey

fun EntryProviderScope<NavKey>.confirm(
    finishAction: FinishConfirmAction,
    onBuy: AssetIdAction,
    cancelAction: CancelAction,
) {
    entry<ConfirmRoute>(
        metadata = { key -> routeArguments(paramsArgument(key.params)) },
    ) { key ->
        val params = remember(key.params) { ConfirmParams.unpack(key.params) }
        ConfirmScreen(
            params = params,
            cancelAction = cancelAction,
            onBuy = onBuy,
            finishAction = finishAction,
        )
    }
}
