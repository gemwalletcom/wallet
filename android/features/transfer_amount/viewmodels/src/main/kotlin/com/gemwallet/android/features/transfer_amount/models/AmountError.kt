package com.gemwallet.android.features.transfer_amount.models

sealed class AmountError : Exception() {
    object None : AmountError()

    object Required : AmountError()


    object IncorrectAmount : AmountError()


    class InsufficientBalance(val assetSymbol: String) : AmountError()


    class MinimumValue(val minimumValue: String) : AmountError()


    class Unknown(val data: String) : AmountError()

    object NoValidatorSelected : AmountError()

    object NoDelegationSelected : AmountError()
}