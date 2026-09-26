package com.gemwallet.android.ui.navigation.routes

import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.assets.presents.select.SelectSendScreen
import com.gemwallet.android.features.assets.viewmodels.select.SendPaymentViewModel
import com.gemwallet.android.features.recipient.presents.RecipientScreen
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
import com.wallet.core.primitives.NFTAsset
import kotlinx.serialization.Contextual
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemPaymentRecipient

@Serializable
data class RecipientInputRoute(val assetId: AssetId, val nft: NFTAsset? = null, val payment: @Contextual GemPaymentRecipient? = null) : NavKey

@Serializable
data class SendSelectRoute(val payment: @Contextual GemPaymentRecipient? = null, val chains: List<Chain> = emptyList()) : NavKey

fun EntryProviderScope<NavKey>.recipientInput(navigator: WalletNavigator, cancelAction: CancelAction, amountAction: AmountTransactionAction, confirmAction: ConfirmTransactionAction) {
    entry<SendSelectRoute> { key ->
        val paymentViewModel: SendPaymentViewModel = hiltViewModel()
        SelectSendScreen(
            chains = key.chains,
            onCancel = cancelAction::invoke,
            onSelect = { assetId ->
                paymentViewModel.onSelect(assetId, key.payment, { id, payment -> navigator.openRecipient(id, payment) }, amountAction, confirmAction)
            },
        )
    }

    entry<RecipientInputRoute>(
        metadata = { key ->
            routeArguments(
                assetIdArgument(key.assetId),
                RouteArgument.Nft to key.nft?.packRoutePayload(),
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
