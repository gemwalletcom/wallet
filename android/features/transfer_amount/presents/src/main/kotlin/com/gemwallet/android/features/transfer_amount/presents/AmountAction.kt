package com.gemwallet.android.features.transfer_amount.presents

internal sealed interface AmountAction {
    data object Next : AmountAction
    data class SetAmount(val amount: String) : AmountAction
    data object SwitchInputType : AmountAction
    data object SetMaxAmount : AmountAction
    data object Cancel : AmountAction
}
