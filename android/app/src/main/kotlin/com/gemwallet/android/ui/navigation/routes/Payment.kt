package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.features.payment.presents.PaymentScreen
import com.gemwallet.android.ui.models.actions.CancelAction
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PaymentProviderName
import kotlinx.serialization.Serializable

@Serializable
data class PaymentRoute(val provider: PaymentProviderName, val paymentId: String) : NavKey

fun EntryProviderScope<NavKey>.payment(
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    cancelAction: CancelAction,
) {
    entry<PaymentRoute> { key ->
        PaymentScreen(
            provider = key.provider,
            paymentId = key.paymentId,
            onAcquireAsset = onAcquireAsset,
            onCancel = { cancelAction() },
        )
    }
}
