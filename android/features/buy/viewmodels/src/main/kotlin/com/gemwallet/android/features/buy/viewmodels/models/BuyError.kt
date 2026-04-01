package com.gemwallet.android.features.buy.viewmodels.models

sealed interface BuyError {
    data object EmptyAmount : BuyError

    data object MinimumAmount : BuyError

    data object MaximumAmount : BuyError

    data object QuoteNotAvailable : BuyError

    data object ValueIncorrect : BuyError

    data object InsufficientBalance : BuyError
}