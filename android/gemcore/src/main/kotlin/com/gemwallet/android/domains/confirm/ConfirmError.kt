package com.gemwallet.android.domains.confirm

import com.gemwallet.android.model.GemNetworkError
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain

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
    class InsufficientFee(val chain: Chain, val requirement: BalanceRequirement) : ConfirmError()
    class MinimumAccountBalanceTooLow(val asset: Asset, val requirement: BalanceRequirement) : ConfirmError()
    class BroadcastError(val details: String) : ConfirmError()
    class NetworkError(val error: GemNetworkError) : ConfirmError()
    class DustThreshold(val chain: Chain) : ConfirmError()
    data object ScanTransactionMalicious : ConfirmError()
    class ScanTransactionMemoRequired(val symbol: String) : ConfirmError()
}
