package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.confirm.presents.AcquireAssetAction
import com.gemwallet.android.features.payment.presents.PaymentScreen
import com.gemwallet.android.ui.models.actions.CancelAction
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PaymentLink
import kotlinx.serialization.Serializable

@Serializable
data class PaymentRoute(val link: PaymentLink) : NavKey

fun EntryProviderScope<NavKey>.payment(
    onAcquireAsset: (AcquireAssetAction, AssetId) -> Unit,
    cancelAction: CancelAction,
) {
    entry<PaymentRoute> { key ->
        PaymentScreen(
            link = key.link,
            onAcquireAsset = onAcquireAsset,
            onCancel = { cancelAction() },
        )
    }
}
