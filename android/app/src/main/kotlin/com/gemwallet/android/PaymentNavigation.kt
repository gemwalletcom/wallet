package com.gemwallet.android

import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.navigation.routes.AmountRoute
import com.gemwallet.android.ui.navigation.routes.ConfirmTransferRoute
import com.gemwallet.android.ui.navigation.routes.PaymentVerificationRoute
import com.gemwallet.android.ui.navigation.routes.RecipientRoute
import com.gemwallet.android.ui.navigation.routes.SendSelectRoute
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemPaymentServiceInterface
import uniffi.gemstone.GemPaymentTarget
import uniffi.gemstone.Payment
import javax.inject.Inject

class PaymentNavigation @Inject constructor(private val getSession: GetSession, private val paymentService: GemPaymentServiceInterface) {

    suspend fun routes(payment: Payment): List<NavKey> {
        val wallet = getSession().first()?.wallet ?: return emptyList()
        return when (val target = paymentService.prepare(payment, wallet.toGem())) {
            is GemPaymentTarget.Confirm -> listOfNotNull(target.transfer.pack()?.let(::ConfirmTransferRoute))

            is GemPaymentTarget.Verify -> listOf(PaymentVerificationRoute(target.url, target.link))

            is GemPaymentTarget.Amount -> listOfNotNull(
                target.asset.id.toAssetId()?.let { AmountParams.Transfer(it, target.payment).pack() }?.let(::AmountRoute),
            )

            is GemPaymentTarget.Recipient -> listOfNotNull(
                target.asset.id.toAssetId()?.let { RecipientRoute(it, payment = target.payment) },
            )

            is GemPaymentTarget.SelectAsset -> listOf(SendSelectRoute(target.payment, target.chains.map { it.requireChain() }))

            GemPaymentTarget.Unsupported -> emptyList()
        }
    }
}
