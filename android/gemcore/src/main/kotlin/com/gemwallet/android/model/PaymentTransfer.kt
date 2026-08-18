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
        val assetId = request.assetId
        if (assetId != null && assetId != assetInfo.asset.id) {
            return PaymentDestination.Unsupported
        }
        val address = assetInfo.asset.chain.checksumAddress(request.address)

        return when (val params = confirmParams(request, address)) {
            null -> PaymentDestination.Recipient(assetInfo.asset.id, request)
            else -> PaymentDestination.Confirm(params)
        }
    }

    private fun confirmParams(request: PaymentRequest, address: String): ConfirmParams? {
        val asset = assetInfo.asset
        val owner = assetInfo.owner ?: return null
        val value = transferValue(request) ?: return null

        if (!asset.chain.isValidAddress(address)) {
            return null
        }
        if (asset.chain.isMemoSupport() && request.memo.isNullOrEmpty()) {
            return null
        }
        return ConfirmParams.Builder(asset, owner, value)
            .transfer(DestinationAddress(address), request.memo)
    }

    private fun transferValue(request: PaymentRequest): BigInteger? =
        when (val amount = request.amount) {
            is PaymentAmount.ExactValue -> Crypto.exact(amount.content, assetInfo.asset.decimals)?.atomicValue
            is PaymentAmount.AtomicValue -> amount.content.toBigIntegerOrNull()
            null -> null
        }
}
