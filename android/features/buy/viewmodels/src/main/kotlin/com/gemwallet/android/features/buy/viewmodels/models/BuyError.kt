package com.gemwallet.android.features.buy.viewmodels.models

import com.gemwallet.android.model.GemNetworkError

sealed interface BuyError {
    data object EmptyAmount : BuyError

    data class MinimumAmount(val minimum: Int) : BuyError

    data class MaximumAmount(val maximum: Int) : BuyError

    data object QuoteNotAvailable : BuyError

    data class QuoteRequestFailed(val networkError: GemNetworkError?) : BuyError

    data object ValueIncorrect : BuyError

    data object InsufficientBalance : BuyError
}
