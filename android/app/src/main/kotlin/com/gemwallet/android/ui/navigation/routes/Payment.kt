package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.features.payment.presents.PaymentScene
import com.gemwallet.android.ui.models.actions.CancelAction
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemPaymentProviderName

@Serializable
data class PaymentRoute(val provider: String, val paymentId: String) : NavKey

fun EntryProviderScope<NavKey>.payment(
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    cancelAction: CancelAction,
) {
    entry<PaymentRoute> { key ->
        PaymentScene(
            provider = GemPaymentProviderName.valueOf(key.provider),
            paymentId = key.paymentId,
            onAcquireAsset = onAcquireAsset,
            onCancel = { cancelAction() },
        )
    }
}
