package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.asset_select.presents.views.SelectSendScreen
import com.gemwallet.android.features.recipient.presents.RecipientScreen
import com.gemwallet.android.model.PaymentRecipient
import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.WalletNavigator
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import kotlinx.serialization.Serializable

@Serializable
data class RecipientInputRoute(
    val assetId: AssetId,
    val nftAssetId: String?,
    val payment: PaymentRecipient? = null,
) : NavKey

@Serializable
data class SendSelectRoute(
    val payment: PaymentRecipient? = null,
    val chains: List<Chain> = emptyList(),
) : NavKey

fun EntryProviderScope<NavKey>.recipientInput(
    navigator: WalletNavigator,
    cancelAction: CancelAction,
    amountAction: AmountTransactionAction,
    confirmAction: ConfirmTransactionAction,
) {
    entry<SendSelectRoute> { key ->
        SelectSendScreen(
            chains = key.chains,
            onCancel = cancelAction::invoke,
            onSelect = { navigator.openRecipient(it, key.payment) },
        )
    }

    entry<RecipientInputRoute>(
        metadata = { key ->
            routeArguments(
                assetIdArgument(key.assetId),
                RouteArgument.NftAssetId to key.nftAssetId,
                RouteArgument.Payment to key.payment?.packRoutePayload(),
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
