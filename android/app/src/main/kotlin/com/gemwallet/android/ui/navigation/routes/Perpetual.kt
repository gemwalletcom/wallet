package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.perpetuals.presents.market.PerpetualMarketNavScreen
import com.gemwallet.android.features.perpetuals.presents.position.PerpetualPositionNavScreen
import com.gemwallet.android.features.transfer.presents.confirm.ConfirmScreen
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.AcquireAssetAction
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionId
import kotlinx.serialization.Serializable

@Serializable
data object PerpetualRoute : NavKey

@Serializable
data class PerpetualPositionRoute(val assetId: AssetId) : NavKey

fun EntryProviderScope<NavKey>.perpetualScreen(
    onCancel: () -> Unit,
    onOpenPerpetualDetails: AssetIdAction,
    onOpenPortfolio: () -> Unit,
    amountAction: AmountTransactionAction,
    confirmAction: ConfirmTransactionAction,
    onTransaction: (TransactionId) -> Unit,
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
) {
    entry<PerpetualRoute> {
        PerpetualMarketNavScreen(
            onOpenPerpetualDetails = onOpenPerpetualDetails,
            onOpenPortfolio = onOpenPortfolio,
            amountAction = amountAction,
            onCancel = onCancel,
        )
    }

    entry<PerpetualPositionRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        PerpetualPositionNavScreen(
            amountAction = amountAction,
            confirmAction = confirmAction,
            onClose = onCancel,
            onTransaction = onTransaction,
            confirmContent = { input, finishAction, cancelAction, onOpenAddress ->
                ConfirmScreen(
                    input = input,
                    cancelAction = cancelAction,
                    finishAction = finishAction,
                    onAcquireAsset = onAcquireAsset,
                    onOpenAddress = onOpenAddress,
                    handleSystemBack = true,
                )
            },
        )
    }
}
