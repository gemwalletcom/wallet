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
    class InsufficientBalance(val chainTitle: String) : ConfirmError()
    class InsufficientFee(val chain: Chain) : ConfirmError()
    class MinimumAccountBalanceTooLow(val asset: Asset, val required: Long) : ConfirmError()
    class BroadcastError(val details: String) : ConfirmError()
    class NetworkError(val error: GemNetworkError) : ConfirmError()
    class DustThreshold(val chain: Chain) : ConfirmError()
}
