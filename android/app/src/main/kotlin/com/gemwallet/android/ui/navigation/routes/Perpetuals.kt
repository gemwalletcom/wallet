package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.assets.presents.select.SelectDepositScreen
import com.gemwallet.android.features.perpetuals.presents.perpetual.PerpetualScreen
import com.gemwallet.android.features.perpetuals.presents.perpetuals.PerpetualsScreen
import com.gemwallet.android.features.transfer.presents.confirm.ConfirmTransferScreen
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.GetAssetAction
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionId
import kotlinx.serialization.Serializable

@Serializable
data object PerpetualsRoute : NavKey

@Serializable
data class PerpetualRoute(val assetId: AssetId) : NavKey

@Serializable
data class PerpetualDepositSelectRoute(val assetIds: List<AssetId>) : NavKey

fun EntryProviderScope<NavKey>.perpetualsScreen(
    onCancel: () -> Unit,
    onOpenPerpetual: AssetIdAction,
    onOpenPortfolio: () -> Unit,
    amountAction: AmountTransactionAction,
    onSelectDepositAsset: (List<AssetId>) -> Unit,
    confirmAction: ConfirmTransactionAction,
    onTransaction: (TransactionId) -> Unit,
    onGetAsset: (GetAssetAction, AssetId) -> Unit,
) {
    entry<PerpetualsRoute> {
        PerpetualsScreen(
            onOpenPerpetual = onOpenPerpetual,
            onOpenPortfolio = onOpenPortfolio,
            amountAction = amountAction,
            onSelectDepositAsset = onSelectDepositAsset,
            onCancel = onCancel,
        )
    }

    entry<PerpetualDepositSelectRoute> { key ->
        SelectDepositScreen(
            assetIds = key.assetIds,
            onCancel = onCancel,
            onSelect = { amountAction(AmountParams.Deposit(it)) },
        )
    }

    entry<PerpetualRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        PerpetualScreen(
            amountAction = amountAction,
            confirmAction = confirmAction,
            onClose = onCancel,
            onTransaction = onTransaction,
            confirmContent = { input, finishAction, cancelAction, onOpenAddress ->
                ConfirmTransferScreen(
                    input = input,
                    cancelAction = cancelAction,
                    finishAction = finishAction,
                    onGetAsset = onGetAsset,
                    onOpenAddress = onOpenAddress,
                    handleSystemBack = true,
                )
            },
        )
    }
}
