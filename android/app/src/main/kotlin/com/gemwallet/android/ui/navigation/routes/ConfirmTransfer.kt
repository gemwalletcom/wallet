package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.remember
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.confirm.unpackConfirmTransferInput
import com.gemwallet.android.features.assets.presents.select.SelectPaymentScreen
import com.gemwallet.android.features.transfer.presents.confirm.ConfirmTransferScreen
import com.gemwallet.android.features.transfer.presents.confirm.PaymentVerificationScreen
import com.gemwallet.android.features.transfer.viewmodels.confirm.models.GetAssetAction
import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.WalletNavigator
import com.gemwallet.android.ui.navigation.paramsArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Contextual
import kotlinx.serialization.Serializable
import uniffi.gemstone.PaymentLink

@Serializable
data class ConfirmTransferRoute(val params: String) : NavKey

@Serializable
data class PaymentSelectRoute(val assetIds: List<AssetId>) : NavKey

@Serializable
data class PaymentVerificationRoute(val url: String, val link: @Contextual PaymentLink) : NavKey

fun EntryProviderScope<NavKey>.confirmTransfer(navigator: WalletNavigator, finishAction: FinishConfirmAction, onGetAsset: (GetAssetAction, AssetId) -> Unit, cancelAction: CancelAction) {
    entry<ConfirmTransferRoute>(
        metadata = { key -> routeArguments(paramsArgument(key.params)) },
    ) { key ->
        val input = remember(key.params) { unpackConfirmTransferInput(key.params) }
        ConfirmTransferScreen(
            input = input,
            cancelAction = cancelAction,
            onGetAsset = onGetAsset,
            paymentAsset = navigator.paymentSelection(key),
            onPaymentAssetConsumed = { navigator.clearPaymentSelection(key) },
            onSelectPaymentAsset = navigator::openPaymentSelect,
            onOpenAddress = navigator::openAddress,
            finishAction = finishAction,
        )
    }

    entry<PaymentSelectRoute> { key ->
        SelectPaymentScreen(
            assetIds = key.assetIds,
            onCancel = cancelAction::invoke,
            onSelect = navigator::finishPaymentSelect,
        )
    }

    entry<PaymentVerificationRoute>(
        metadata = { key -> routeArguments(RouteArgument.Url to key.url, RouteArgument.PaymentLink to key.link.packRoutePayload()) },
    ) {
        PaymentVerificationScreen(
            onCancel = cancelAction::invoke,
            onConfirm = navigator::replaceWithConfirmTransfer,
        )
    }
}
