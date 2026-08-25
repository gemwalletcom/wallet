package com.gemwallet.android

import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.application.asset_select.coordinators.GetSelectAssetsInfo
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.PaymentDestination
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.gemwallet.android.ui.navigation.routes.RecipientInputRoute
import com.gemwallet.android.ui.navigation.routes.SendSelectRoute
import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentRequest
import kotlinx.coroutines.flow.first
import uniffi.gemstone.ChainAddress
import uniffi.gemstone.PaymentServiceInterface
import java.math.BigInteger
import javax.inject.Inject

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
        val addresses = assets.mapNotNull { it.owner }
            .distinctBy { it.chain }
            .map { ChainAddress(chain = it.chain.string, address = it.address) }
        val payment = paymentService.load(link.toGem(), addresses)
        val assetInfo = assets.firstOrNull { assetInfo ->
            val owner = assetInfo.owner ?: return@firstOrNull false
            assetInfo.asset.id.tokenId == null &&
                owner.chain.string == payment.account.chain &&
                owner.address == payment.account.address
        } ?: return emptyList()
        val account = assetInfo.owner ?: return emptyList()
        val params = ConfirmParams.TransferParams.Generic(
            asset = assetInfo.asset,
            from = account,
            amount = BigInteger.ZERO,
            destination = DestinationAddress(""),
            memo = payment.memo,
            inputType = ConfirmParams.TransferParams.InputType.EncodeTransaction,
            isSendable = true,
            metadata = payment.merchant.toPrimitives(),
            data = payment.transaction,
            gasLimit = null,
            decodedTransactionType = payment.transactionType.toPrimitives(),
        )
        return listOfNotNull(params.pack()?.let(::ConfirmRoute))
    }
}
