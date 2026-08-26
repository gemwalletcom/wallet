package com.gemwallet.android

import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.asset_select.coordinators.GetSelectAssetsInfo
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.PaymentDestination
import com.gemwallet.android.model.toPaymentWalletAsset
import com.gemwallet.android.model.toTransferParams
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.gemwallet.android.ui.navigation.routes.RecipientInputRoute
import com.gemwallet.android.ui.navigation.routes.SendSelectRoute
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentRequest
import com.wallet.core.primitives.TransactionType
import java.math.BigInteger
import javax.inject.Inject
import kotlinx.coroutines.flow.first
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.PaymentServiceInterface
import uniffi.gemstone.paymentDecodedTransfer

class PaymentNavigation @Inject constructor(
    private val getSelectAssetsInfo: GetSelectAssetsInfo,
    private val paymentService: PaymentServiceInterface,
) {

    suspend fun routes(payment: Payment): List<NavKey> = when (payment) {
        is Payment.Request -> requestRoutes(payment.content)
        is Payment.Link -> linkRoutes(payment.content)
    }

    private suspend fun requestRoutes(request: PaymentRequest): List<NavKey> =
        when (val destination = PaymentDestination.from(request, getSelectAssetsInfo().first())) {
            PaymentDestination.Unsupported -> emptyList()
            is PaymentDestination.Confirm -> listOfNotNull(destination.params.pack()?.let(::ConfirmRoute))
            is PaymentDestination.Recipient -> listOf(
                RecipientInputRoute(destination.assetId, nftAssetId = null, payment = destination.request)
            )
            is PaymentDestination.SelectAsset -> listOf(SendSelectRoute(destination.request, destination.chains))
        }

    private suspend fun linkRoutes(link: PaymentLink): List<NavKey> {
        val assets = getSelectAssetsInfo().first()
        val accounts = assets.mapNotNull { it.owner }.distinctBy { it.chain }
        val payment = paymentService.load(
            link.toJson(),
            accounts.map { ChainAddress(chain = it.chain, address = it.address).toJson() },
        )
        val paymentAccount = payment.account.decodeJson<ChainAddress>()
        val account = accounts.firstOrNull {
            it.chain == paymentAccount.chain && it.address == paymentAccount.address
        } ?: return emptyList()
        val transfer = payment.request?.let { request ->
            val decoded = request.decodeJson<PaymentRequest>()
            assets.firstOrNull { it.asset.id == decoded.assetId }
                ?.let { paymentDecodedTransfer(request, it.toPaymentWalletAsset())?.toTransferParams(assets) }
        } ?: ConfirmParams.Builder(account.chain.asset(), account, BigInteger.ZERO)
            .transfer(DestinationAddress(""), payment.memo)
        val params = ConfirmParams.TransferParams.Generic(
            asset = transfer.asset,
            from = transfer.from,
            amount = transfer.amount,
            destination = transfer.destination,
            memo = transfer.memo,
            inputType = ConfirmParams.TransferParams.InputType.EncodeTransaction,
            isSendable = true,
            metadata = payment.merchant.decodeJson(),
            data = payment.transaction,
            gasLimit = null,
            decodedTransactionType = payment.transactionType.decodeJson(),
        )
        return listOfNotNull(params.pack()?.let(::ConfirmRoute))
    }
}
