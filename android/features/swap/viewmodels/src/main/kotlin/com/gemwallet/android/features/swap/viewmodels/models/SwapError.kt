package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.features.swap.viewmodels.models.SwapError.InputAmountTooSmall
import com.gemwallet.android.features.swap.viewmodels.models.SwapError.NoQuote
import com.gemwallet.android.features.swap.viewmodels.models.SwapError.NotSupportedAsset
import com.wallet.core.primitives.Asset
import uniffi.gemstone.SwapperException
import java.math.BigDecimal
import java.math.BigInteger

sealed class SwapError : Throwable() {
    object None : SwapError()
    object NoQuote : SwapError()
    object NotSupportedAsset : SwapError()
    class InputAmountTooSmall(val minAmount: String?) : SwapError() {
        fun getValue(asset: Asset): BigDecimal = try {
            val value = minAmount ?: throw IllegalArgumentException()
            Crypto(BigInteger(value)).value(asset.decimals)
        } catch (_: Throwable) {
            null
        } ?: BigDecimal.ZERO

        fun getFormattedValue(asset: Asset): String = try {
            val value = minAmount ?: throw IllegalArgumentException()
            ValueFormatter(style = ValueFormatter.Style.Full).string(BigInteger(value), asset)
        } catch (_: Throwable) {
            null
        } ?: ""
    }
    data class InsufficientBalance(val symbol: String) : SwapError()
    data class Unknown(val data: String) : SwapError()

    companion object
}

fun SwapError.Companion.toError(err: Throwable) = when (err) {
    is SwapperException.NotSupportedAsset,
    is SwapperException.NotSupportedChain -> NotSupportedAsset
    is SwapperException.ComputeQuoteException,
    is SwapperException.NoQuoteAvailable,
    is SwapperException.InvalidRoute,
    is SwapperException.NoAvailableProvider,
    is SwapperException.TransactionException -> NoQuote
    is SwapperException.InputAmountException -> InputAmountTooSmall(err.minAmount)
    else -> SwapError.Unknown(err.localizedMessage ?: err.message ?: "")
}
