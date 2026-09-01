package com.gemwallet.android

import uniffi.gemstone.GemRecipient
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.asset_select.cases.GetSelectAssetsInfo
import com.gemwallet.android.ext.asset
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.confirmInput
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.domains.confirm.toTransactionData
import com.gemwallet.android.domains.confirm.transfer
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransferDataOutputType
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemTransferDataExtra
import com.gemwallet.android.model.PaymentDestination
import com.gemwallet.android.model.toPaymentWalletAsset
import com.gemwallet.android.model.toConfirmInput
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.gemwallet.android.ui.navigation.routes.RecipientInputRoute
import com.gemwallet.android.ui.navigation.routes.SendSelectRoute
import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentRequest
import java.math.BigInteger
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
                RecipientInputRoute(destination.assetId, nftAssetId = null, payment = destination.request)
            )
            is PaymentDestination.SelectAsset -> listOf(SendSelectRoute(destination.request, destination.chains))
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
        val decoded = payment.request?.let { request ->
            val decodedRequest = request.decodeJson<PaymentRequest>()
            assets.firstOrNull { it.asset.id == decodedRequest.assetId }
                ?.let { paymentService.decodedTransfer(request, it.toPaymentWalletAsset())?.toConfirmInput(assets) }
        }
        val transfer = decoded?.transfer ?: GemTransferData(
            inputType = GemTransactionInputType.transfer(account.chain.asset()),
            recipient = GemRecipient(address = "", memo = payment.memo),
            value = BigInteger.ZERO.toString(),
        )
        val input = GemTransferData(
            inputType = GemTransactionInputType.Generic(
                asset = transfer.inputType.asset.toGem(),
                metadata = payment.merchant,
                extra = GemTransferDataExtra(
                    to = transfer.recipient.address,
                    gasLimit = null,
                    gasPrice = null,
                    data = payment.transaction.toTransactionData(),
                    outputType = TransferDataOutputType.EncodedTransaction.toJson(),
                    outputAction = TransferDataOutputAction.Send.toJson(),
                    transactionType = payment.transactionType,
                    approval = null,
                ),
            ),
            recipient = transfer.recipient,
            value = transfer.value,
        ).confirmInput(decoded?.from?.toPrimitives() ?: account)
        return listOfNotNull(transferService.pack(input)?.let(::ConfirmRoute))
    }
}
