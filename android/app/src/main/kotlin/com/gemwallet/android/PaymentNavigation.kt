package com.gemwallet.android

import com.gemwallet.android.ext.toGem
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.model.PaymentDestination
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.gemwallet.android.ui.navigation.routes.RecipientInputRoute
import com.gemwallet.android.ui.navigation.routes.SendSelectRoute
import javax.inject.Inject
import kotlinx.coroutines.flow.first
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemPaymentLoad
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.Payment
import uniffi.gemstone.PaymentLink
import uniffi.gemstone.PaymentRequest

class PaymentNavigation @Inject constructor(
    private val getWalletAssets: GetWalletAssets,
    private val paymentService: GemPaymentService,
) {

    suspend fun routes(payment: Payment): List<NavKey> = when (payment) {
        is Payment.Request -> requestRoutes(payment.request)
        is Payment.Link -> linkRoutes(payment.link)
    }

    private suspend fun requestRoutes(request: PaymentRequest): List<NavKey> =
        when (val destination = PaymentDestination.from(request, getWalletAssets().first(), paymentService)) {
            PaymentDestination.Unsupported -> emptyList()
            is PaymentDestination.Confirm -> listOfNotNull(destination.transfer.pack()?.let(::ConfirmRoute))
            is PaymentDestination.Recipient -> listOf(
                RecipientInputRoute(destination.assetId, nftAssetId = null, payment = destination.payment)
            )
            is PaymentDestination.SelectAsset -> listOf(SendSelectRoute(destination.payment, destination.chains))
        }

    private suspend fun linkRoutes(link: PaymentLink): List<NavKey> {
        val assets = getWalletAssets().first()
        val accounts = assets.mapNotNull { it.owner }.distinctBy { it.chain }
        val load = paymentService.load(
            link,
            accounts.map { ChainAddress(chain = it.chain, address = it.address).toGem() },
        )
        return when (load) {
            is GemPaymentLoad.Sign -> listOfNotNull(load.transfer.pack()?.let(::ConfirmRoute))
            is GemPaymentLoad.Verify -> emptyList()
        }
    }
}
