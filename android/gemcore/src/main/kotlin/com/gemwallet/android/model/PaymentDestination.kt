package com.gemwallet.android.model

import com.gemwallet.android.domains.asset.chain
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentRequest

sealed interface PaymentDestination {

    sealed interface Transfer : PaymentDestination

    data object Unsupported : Transfer

    data class Confirm(val params: ConfirmParams) : Transfer

    data class Recipient(val assetId: AssetId, val request: PaymentRequest) : Transfer

    data class SelectAsset(val request: PaymentRequest, val chains: List<Chain>) : PaymentDestination

    companion object {
        fun from(request: PaymentRequest, assets: List<AssetInfo>): PaymentDestination =
            when (val payable = PaymentAsset.from(request, assets)) {
                PaymentAsset.Unsupported -> Unsupported
                is PaymentAsset.Single -> PaymentTransfer(payable.assetInfo).destination(request)
                is PaymentAsset.Choice -> SelectAsset(
                    request = request,
                    chains = payable.assets.map { it.asset.chain }.distinct(),
                )
            }
    }
}
