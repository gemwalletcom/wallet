package com.gemwallet.android.model

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.isValidAddress
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
        fun from(request: PaymentRequest, assets: List<AssetInfo>): PaymentDestination {
            val payable = payableAssets(request, assets)

            return when (payable.size) {
                0 -> Unsupported
                1 -> PaymentTransfer(payable.first()).destination(request)
                else -> SelectAsset(request, payable.map { it.asset.chain }.distinct())
            }
        }

        private fun payableAssets(request: PaymentRequest, assets: List<AssetInfo>): List<AssetInfo> =
            when (val assetId = request.assetId) {
                null -> assets.filter { it.asset.chain.isValidAddress(request.address) }
                else -> assets.filter { it.asset.id == assetId }
            }
    }
}
