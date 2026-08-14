package com.gemwallet.android.model

import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.checksumAddress
import com.gemwallet.android.ext.isMemoSupport
import com.gemwallet.android.ext.isValidAddress
import com.wallet.core.primitives.PaymentAmount
import com.wallet.core.primitives.PaymentRequest
import java.math.BigInteger

class PaymentTransfer(private val assetInfo: AssetInfo) {

    fun destination(request: PaymentRequest): PaymentDestination.Transfer {
        if (!isSameAsset(request)) {
            return PaymentDestination.Unsupported
        }
        val address = assetInfo.asset.chain.checksumAddress(request.address)

        return when (val params = confirmParams(request, address)) {
            null -> PaymentDestination.Recipient(assetInfo.asset.id, request)
            else -> PaymentDestination.Confirm(params)
        }
    }

    private fun isSameAsset(request: PaymentRequest): Boolean {
        val assetId = request.assetId ?: return true
        return assetId == assetInfo.asset.id
    }

    private fun confirmParams(request: PaymentRequest, address: String): ConfirmParams? {
        val asset = assetInfo.asset
        val owner = assetInfo.owner ?: return null
        val value = transferValue(request) ?: return null

        if (!asset.chain.isValidAddress(address) || needsMemoReview(request)) {
            return null
        }
        return ConfirmParams.Builder(asset, owner, value)
            .transfer(DestinationAddress(address), request.memo)
    }

    private fun needsMemoReview(request: PaymentRequest): Boolean =
        assetInfo.asset.chain.isMemoSupport() && request.memo != null

    private fun transferValue(request: PaymentRequest): BigInteger? =
        when (val amount = request.amount) {
            is PaymentAmount.ExactValue -> Crypto.exact(amount.content, assetInfo.asset.decimals)?.atomicValue
            is PaymentAmount.AtomicValue -> amount.content.toBigIntegerOrNull()
            null -> null
        }
}
