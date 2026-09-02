package com.gemwallet.android.model

import com.gemwallet.android.domains.confirm.confirmInput
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.GemRecipientSerializer
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentRequest
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemPaymentConfirmTransfer
import uniffi.gemstone.GemPaymentDestination
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemPaymentWalletAsset
import uniffi.gemstone.GemRecipient

@Serializable
data class PaymentRecipient(
    val recipient: @Serializable(with = GemRecipientSerializer::class) GemRecipient,
    val amount: String? = null,
)

sealed interface PaymentDestination {

    sealed interface Transfer : PaymentDestination

    data object Unsupported : Transfer

    data class Confirm(val input: GemConfirmInput) : Transfer

    data class Recipient(val assetId: AssetId, val payment: PaymentRecipient) : Transfer

    data class SelectAsset(val payment: PaymentRecipient, val chains: List<Chain>) : PaymentDestination

    companion object {
        fun from(request: PaymentRequest, assets: List<AssetInfo>, paymentService: GemPaymentService): PaymentDestination =
            when (val destination = paymentService.destination(request.toJson(), assets.map { it.toPaymentWalletAsset() })) {
                is GemPaymentDestination.Confirm -> destination.transfer.toConfirmInput(assets, paymentService)?.let(::Confirm) ?: Unsupported
                is GemPaymentDestination.Recipient -> Recipient(destination.assetId.toAssetId()!!, PaymentRecipient(destination.recipient, destination.amount))
                is GemPaymentDestination.SelectAsset -> SelectAsset(
                    PaymentRecipient(destination.recipient, destination.amount),
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

fun GemPaymentConfirmTransfer.toConfirmInput(assets: List<AssetInfo>, paymentService: GemPaymentService): GemConfirmInput? {
    val assetInfo = assets.firstOrNull { it.asset.id.toIdentifier() == assetId } ?: return null
    val owner = assetInfo.owner ?: return null

    return paymentService.transferData(this, assetInfo.asset.toGem()).confirmInput(owner)
}
