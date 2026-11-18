package com.gemwallet.android.features.transfer.presents.amount

internal sealed interface AmountAction {
    data object Next : AmountAction
    data object SwitchInputType : AmountAction
    data object SetMaxAmount : AmountAction
    data object Buy : AmountAction
    data object Cancel : AmountAction
}
