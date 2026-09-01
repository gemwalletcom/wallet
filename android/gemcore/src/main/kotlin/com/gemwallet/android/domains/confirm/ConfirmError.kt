package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.GemNetworkError
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemTransferAmountException
import java.math.BigInteger

sealed class ConfirmError : Exception() {
    data object None : ConfirmError()
    data object Init : ConfirmError()
    data object PreloadError : ConfirmError()
    data object TransactionIncorrect : ConfirmError()
    data object RecipientEmpty : ConfirmError()
    data object SignFail : ConfirmError()
    class InsufficientBalance(
        val asset: Asset,
        val requirement: BalanceRequirement,
    ) : ConfirmError()
    class InsufficientFee(val chain: Chain, val requirement: BalanceRequirement?) : ConfirmError()
    class MinimumAccountBalanceTooLow(val asset: Asset, val requirement: BalanceRequirement) : ConfirmError()
    class BroadcastError(val details: String) : ConfirmError()
    class NetworkError(val error: GemNetworkError) : ConfirmError()
    class DustThreshold(val chain: Chain) : ConfirmError()
    data object ScanTransactionMalicious : ConfirmError()
    class ScanTransactionMemoRequired(val symbol: String) : ConfirmError()
}

fun GemTransferAmountException.toConfirmError(asset: Asset, feeAsset: Asset): ConfirmError = when (this) {
    is GemTransferAmountException.InsufficientBalance -> ConfirmError.InsufficientBalance(
        asset = amountAsset(assetId, asset, feeAsset),
        requirement = amountRequirement(required, available),
    )
    is GemTransferAmountException.InsufficientNetworkFee -> ConfirmError.InsufficientFee(
        chain = feeAsset.id.chain,
        requirement = amountRequirement(required, available),
    )
    is GemTransferAmountException.MinimumAccountBalanceTooLow -> ConfirmError.MinimumAccountBalanceTooLow(
        asset = amountAsset(assetId, asset, feeAsset),
        requirement = amountRequirement(required, available),
    )
}

private fun amountAsset(assetId: String, asset: Asset, feeAsset: Asset): Asset =
    if (asset.id.toIdentifier() == assetId) asset else feeAsset

private fun amountRequirement(required: String, available: String) = BalanceRequirement(
    required = BigInteger(required),
    available = BigInteger(available),
)
