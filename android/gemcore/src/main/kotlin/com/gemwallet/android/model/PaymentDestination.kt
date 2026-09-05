package com.gemwallet.android.model

import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentRequest
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemPaymentConfirmTransfer
import uniffi.gemstone.GemPaymentDestination
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemPaymentWalletAsset

sealed interface PaymentDestination {

    sealed interface Transfer : PaymentDestination

    data object Unsupported : Transfer

    data class Confirm(val transfer: GemTransferData) : Transfer

    data class Recipient(val assetId: AssetId, val payment: GemPaymentRecipient) : Transfer

    data class SelectAsset(val payment: GemPaymentRecipient, val chains: List<Chain>) : PaymentDestination

    companion object {
        fun from(request: PaymentRequest, assets: List<AssetInfo>, paymentService: GemPaymentService): PaymentDestination =
            when (val destination = paymentService.destination(request.toJson(), assets.map { it.toPaymentWalletAsset() })) {
                is GemPaymentDestination.Confirm -> destination.transfer.toTransferData(assets, paymentService)?.let(::Confirm) ?: Unsupported
                is GemPaymentDestination.Recipient -> Recipient(destination.assetId.toAssetId()!!, destination.payment)
                is GemPaymentDestination.SelectAsset -> SelectAsset(
                    destination.payment,
                    destination.chains.map { chain -> Chain.entries.first { it.string == chain } },
                )
                is GemPaymentDestination.Unsupported -> Unsupported
            }
    }
}

fun AssetInfo.toPaymentWalletAsset(): GemPaymentWalletAsset = GemPaymentWalletAsset(
    assetId = asset.id.toIdentifier(),
    decimals = asset.decimals,
)

fun GemPaymentConfirmTransfer.toTransferData(assets: List<AssetInfo>, paymentService: GemPaymentService): GemTransferData? {
    val assetInfo = assets.firstOrNull { it.asset.id.toIdentifier() == assetId } ?: return null
    return paymentService.transferData(this, assetInfo.asset.toGem())
}
