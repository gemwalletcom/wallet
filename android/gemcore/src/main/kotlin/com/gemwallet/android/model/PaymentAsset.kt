package com.gemwallet.android.model

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.isValidAddress
import com.wallet.core.primitives.PaymentRequest

internal sealed interface PaymentAsset {
    data object Unsupported : PaymentAsset

    data class Single(val assetInfo: AssetInfo) : PaymentAsset

    data class Choice(val assets: List<AssetInfo>) : PaymentAsset

    companion object {
        fun from(request: PaymentRequest, assets: List<AssetInfo>): PaymentAsset {
            val payable = when (val assetId = request.assetId) {
                null -> assets.filter { it.asset.chain.isValidAddress(request.address) }
                else -> assets.filter { it.asset.id == assetId }
            }
            return when (payable.size) {
                0 -> Unsupported
                1 -> Single(payable.first())
                else -> Choice(payable)
            }
        }
    }
}
