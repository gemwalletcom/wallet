package com.gemwallet.android.features.transfer_amount.models

sealed class AmountError : Exception() {
    object None : AmountError()

    object Required : AmountError()

    object Unavailable : AmountError()

    object IncorrectAmount : AmountError()

    object ZeroAmount : AmountError()

    class InsufficientBalance(val assetSymbol: String) : AmountError()

    class InsufficientFeeBalance(val assetName: String) : AmountError()

    class MinimumValue(val minimumValue: String) : AmountError()

    object IncorrectAddress : AmountError()

    class Unknown(val data: String) : AmountError()
}