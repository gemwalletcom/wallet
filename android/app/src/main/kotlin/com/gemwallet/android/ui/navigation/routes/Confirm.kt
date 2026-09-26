package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.remember
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.confirm.unpackConfirmTransferInput
import com.gemwallet.android.features.assets.presents.select.SelectPaymentScreen
import com.gemwallet.android.features.confirm.presents.ConfirmScreen
import com.gemwallet.android.features.confirm.presents.PaymentVerificationScreen
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireAssetAction
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
data class ConfirmRoute(val params: String) : NavKey

@Serializable
data class PaymentSelectRoute(val assetIds: List<AssetId>) : NavKey

@Serializable
data class PaymentVerificationRoute(val url: String, val link: @Contextual PaymentLink) : NavKey

fun EntryProviderScope<NavKey>.confirm(navigator: WalletNavigator, finishAction: FinishConfirmAction, onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit, cancelAction: CancelAction) {
    entry<ConfirmRoute>(
        metadata = { key -> routeArguments(paramsArgument(key.params)) },
    ) { key ->
        val input = remember(key.params) { unpackConfirmTransferInput(key.params) }
        ConfirmScreen(
            input = input,
            cancelAction = cancelAction,
            onAcquireAsset = onAcquireAsset,
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
            onConfirm = navigator::replaceWithConfirm,
        )
    }
}
