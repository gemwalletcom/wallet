package com.gemwallet.android.features.buy.viewmodels.models

sealed interface BuyError {
    data object EmptyAmount : BuyError

    data class MinimumAmount(val minimum: Int) : BuyError

    data class MaximumAmount(val maximum: Int) : BuyError

    data object QuoteNotAvailable : BuyError

    data object ValueIncorrect : BuyError

    data object InsufficientBalance : BuyError
}