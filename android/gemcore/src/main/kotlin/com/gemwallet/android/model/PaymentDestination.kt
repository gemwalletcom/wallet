package com.gemwallet.android.model

import com.gemwallet.android.domains.confirm.confirmInput
import com.gemwallet.android.domains.confirm.transfer
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentRequest
import uniffi.gemstone.GemPaymentConfirmTransfer
import uniffi.gemstone.GemPaymentDestination
import uniffi.gemstone.GemPaymentWalletAsset
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData

sealed interface PaymentDestination {

    sealed interface Transfer : PaymentDestination

    data object Unsupported : Transfer

    data class Confirm(val input: GemConfirmInput) : Transfer

    data class Recipient(val assetId: AssetId, val request: PaymentRequest) : Transfer

    data class SelectAsset(val request: PaymentRequest, val chains: List<Chain>) : PaymentDestination

    companion object {
        fun from(request: PaymentRequest, assets: List<AssetInfo>, paymentService: GemPaymentService): PaymentDestination =
            when (val destination = paymentService.destination(request.toJson(), assets.map { it.toPaymentWalletAsset() })) {
                is GemPaymentDestination.Confirm -> destination.transfer.toConfirmInput(assets)?.let(::Confirm) ?: Unsupported
                is GemPaymentDestination.Recipient -> destination.assetId.toAssetId()?.let { Recipient(it, request) } ?: Unsupported
                is GemPaymentDestination.SelectAsset -> SelectAsset(request, destination.chains.mapNotNull { chain -> Chain.entries.firstOrNull { it.string == chain } })
                is GemPaymentDestination.Unsupported -> Unsupported
            }

        fun transfer(request: PaymentRequest, assetInfo: AssetInfo, paymentService: GemPaymentService): Transfer =
            when (val destination = paymentService.transferDestination(request.toJson(), assetInfo.toPaymentWalletAsset())) {
                is GemPaymentDestination.Confirm -> destination.transfer.toConfirmInput(listOf(assetInfo))?.let(::Confirm) ?: Unsupported
                is GemPaymentDestination.Recipient -> Recipient(assetInfo.asset.id, request)
                is GemPaymentDestination.SelectAsset -> Unsupported
                is GemPaymentDestination.Unsupported -> Unsupported
            }
    }
}

fun AssetInfo.toPaymentWalletAsset(): GemPaymentWalletAsset = GemPaymentWalletAsset(
    assetId = asset.id.toIdentifier(),
    decimals = asset.decimals,
)

fun GemPaymentConfirmTransfer.toConfirmInput(assets: List<AssetInfo>): GemConfirmInput? {
    val assetInfo = assets.firstOrNull { it.asset.id.toIdentifier() == assetId } ?: return null
    val owner = assetInfo.owner ?: return null
    val value = value.toBigIntegerOrNull() ?: return null

    return GemTransferData(
        inputType = GemTransactionInputType.transfer(assetInfo.asset),
        recipient = GemRecipient(address = address, memo = memo, references = references),
        value = value.toString(),
    ).confirmInput(owner)
}
