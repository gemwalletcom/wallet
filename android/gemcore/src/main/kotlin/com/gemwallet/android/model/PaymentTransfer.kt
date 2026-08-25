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
        val params = if (requiresMemo(request)) null else decodedTransfer(request)

        return when (params) {
            null -> PaymentDestination.Recipient(assetInfo.asset.id, request)
            else -> PaymentDestination.Confirm(params)
        }
    }

    fun decodedTransfer(request: PaymentRequest): ConfirmParams.TransferParams? {
        val asset = assetInfo.asset
        val owner = assetInfo.owner ?: return null
        val assetId = request.assetId
        if (assetId != null && assetId != asset.id) {
            return null
        }
        val value = transferValue(request) ?: return null
        val address = asset.chain.checksumAddress(request.address)

        if (!asset.chain.isValidAddress(address)) {
            return null
        }
        return ConfirmParams.Builder(asset, owner, value)
            .transfer(DestinationAddress(address), request.memo, request.references.orEmpty())
    }

    private fun requiresMemo(request: PaymentRequest): Boolean =
        assetInfo.asset.chain.isMemoSupport() && request.memo.isNullOrEmpty()

    private fun transferValue(request: PaymentRequest): BigInteger? =
        when (val amount = request.amount) {
            is PaymentAmount.ExactValue -> Crypto.exact(amount.content, assetInfo.asset.decimals)?.atomicValue
            is PaymentAmount.AtomicValue -> amount.content.toBigIntegerOrNull()
            null -> null
        }
}
