package com.gemwallet.android

import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.asset_select.cases.GetSelectAssetsInfo
import com.gemwallet.android.ext.asset
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.confirmInput
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.PaymentDestination
import com.gemwallet.android.model.toPaymentWalletAsset
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
import uniffi.gemstone.GemPaymentLinkServiceInterface
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemTransferService

class PaymentNavigation @Inject constructor(
    private val getSelectAssetsInfo: GetSelectAssetsInfo,
    private val paymentLinkService: GemPaymentLinkServiceInterface,
    private val paymentService: GemPaymentService,
    private val transferService: GemTransferService,
) {

    suspend fun routes(payment: Payment): List<NavKey> = when (payment) {
        is Payment.Request -> requestRoutes(payment.content)
        is Payment.Link -> linkRoutes(payment.content)
    }

    private suspend fun requestRoutes(request: PaymentRequest): List<NavKey> =
        when (val destination = PaymentDestination.from(request, getSelectAssetsInfo().first(), paymentService)) {
            PaymentDestination.Unsupported -> emptyList()
            is PaymentDestination.Confirm -> listOfNotNull(transferService.pack(destination.input)?.let(::ConfirmRoute))
            is PaymentDestination.Recipient -> listOf(
                RecipientInputRoute(destination.assetId, nftAssetId = null, payment = destination.payment)
            )
            is PaymentDestination.SelectAsset -> listOf(SendSelectRoute(destination.payment, destination.chains))
        }

    private suspend fun linkRoutes(link: PaymentLink): List<NavKey> {
        val assets = getSelectAssetsInfo().first()
        val accounts = assets.mapNotNull { it.owner }.distinctBy { it.chain }
        val payment = paymentLinkService.load(
            link.toJson(),
            accounts.map { ChainAddress(chain = it.chain, address = it.address).toJson() },
        )
        val paymentAccount = payment.account.decodeJson<ChainAddress>()
        val account = accounts.firstOrNull {
            it.chain == paymentAccount.chain && it.address == paymentAccount.address
        } ?: return emptyList()
        val assetInfo = payment.request
            ?.decodeJson<PaymentRequest>()
            ?.let { request -> assets.firstOrNull { it.asset.id == request.assetId } }
        val asset = assetInfo?.asset ?: account.chain.asset()
        val input = paymentService
            .transactionTransferData(payment, asset.toGem())
            .confirmInput(assetInfo?.owner ?: account)
        return listOfNotNull(transferService.pack(input)?.let(::ConfirmRoute))
    }
}
