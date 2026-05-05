package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.recipient.presents.RecipientScreen
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.features.asset_select.presents.views.SelectSendScreen
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable

@Serializable
data class RecipientInputRoute(val assetId: AssetId, val nftAssetId: String?) : NavKey

@Serializable
data object SendSelectRoute : NavKey

fun EntryProviderScope<NavKey>.recipientInput(
    cancelAction: CancelAction,
    recipientAction: AssetIdAction,
    amountAction: AmountTransactionAction,
    confirmAction: ConfirmTransactionAction,
) {
    entry<SendSelectRoute> {
        SelectSendScreen(
            onCancel = cancelAction::invoke,
            onSelect = recipientAction::invoke,
        )
    }

    entry<RecipientInputRoute>(
        metadata = { key ->
            routeArguments(
                assetIdArgument(key.assetId),
                RouteArgument.NftAssetId to key.nftAssetId,
            )
        },
    ) {
        RecipientScreen(
            cancelAction = cancelAction,
            amountAction = amountAction,
            confirmAction = confirmAction,
        )
    }
}
