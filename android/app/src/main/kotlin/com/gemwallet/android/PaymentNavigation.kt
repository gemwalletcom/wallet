package com.gemwallet.android

import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.asset_select.cases.GetSelectAssetsInfo
import com.gemwallet.android.ext.asset
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.PaymentDestination
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.gemwallet.android.ui.navigation.routes.RecipientInputRoute
import com.gemwallet.android.ui.navigation.routes.SendSelectRoute
import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentRequest
import javax.inject.Inject
import kotlinx.coroutines.flow.first
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemAssetsServiceInterface
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemTransferService

class PaymentNavigation @Inject constructor(
    private val getSelectAssetsInfo: GetSelectAssetsInfo,
    private val paymentService: GemPaymentService,
    private val transferService: GemTransferService,
    private val assetsService: GemAssetsServiceInterface,
) {

    suspend fun routes(payment: Payment): List<NavKey> = when (payment) {
        is Payment.Request -> requestRoutes(payment.content)
        is Payment.Link -> linkRoutes(payment.content)
    }

    private suspend fun requestRoutes(request: PaymentRequest): List<NavKey> =
        when (val destination = PaymentDestination.from(request, getSelectAssetsInfo().first(), paymentService)) {
            PaymentDestination.Unsupported -> emptyList()
            is PaymentDestination.Confirm -> listOfNotNull(transferService.pack(destination.transfer)?.let(::ConfirmRoute))
            is PaymentDestination.Recipient -> listOf(
                RecipientInputRoute(destination.assetId, nftAssetId = null, payment = destination.payment)
            )
            is PaymentDestination.SelectAsset -> listOf(SendSelectRoute(destination.payment, destination.chains))
        }

    private suspend fun linkRoutes(link: PaymentLink): List<NavKey> {
        val assets = getSelectAssetsInfo().first()
        val accounts = assets.mapNotNull { it.owner }.distinctBy { it.chain }
        val payment = paymentService.load(
            link.toJson(),
            accounts.map { ChainAddress(chain = it.chain, address = it.address).toJson() },
        )
        val chain = payment.account.decodeJson<ChainAddress>().chain
        val assetId = payment.request?.decodeJson<PaymentRequest>()?.assetId ?: chain.asset().id
        val asset = assetsService.ensureTokenAsset(assetId.toIdentifier())
        val transfer = paymentService.transactionTransferData(payment, asset)
        return listOfNotNull(transferService.pack(transfer)?.let(::ConfirmRoute))
    }
}
