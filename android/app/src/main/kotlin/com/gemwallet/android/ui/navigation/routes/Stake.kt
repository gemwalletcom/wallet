package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.features.stake.presents.EarnScreen
import com.gemwallet.android.features.stake.presents.StakeScreen
import com.gemwallet.android.features.stake.presents.delegation.DelegationScene
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAddress
import kotlinx.serialization.Serializable

@Serializable
data class StakeRoute(val assetId: AssetId) : NavKey

@Serializable
data class EarnRoute(val assetId: AssetId) : NavKey

@Serializable
data class DelegationRoute(val validatorId: String, val delegationId: String) : NavKey

fun EntryProviderScope<NavKey>.stake(onAmount: AmountTransactionAction, onConfirm: (ConfirmTransferInput) -> Unit, onDelegation: (String, String) -> Unit, onOpenAddress: (ChainAddress) -> Unit, onCancel: () -> Unit) {
    entry<StakeRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        StakeScreen(
            amountAction = onAmount,
            onDelegation = onDelegation,
            onConfirm = onConfirm,
            onCancel = onCancel,
        )
    }

    entry<EarnRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        EarnScreen(
            amountAction = onAmount,
            onDelegation = onDelegation,
            onConfirm = onConfirm,
            onCancel = onCancel,
        )
    }

    entry<DelegationRoute>(
        metadata = { key ->
            routeArguments(
                RouteArgument.ValidatorId to key.validatorId,
                RouteArgument.DelegationId to key.delegationId,
            )
        },
    ) {
        DelegationScene(
            onAmount = onAmount,
            onConfirm = onConfirm,
            onOpenAddress = onOpenAddress,
            onCancel = onCancel,
        )
    }
}
