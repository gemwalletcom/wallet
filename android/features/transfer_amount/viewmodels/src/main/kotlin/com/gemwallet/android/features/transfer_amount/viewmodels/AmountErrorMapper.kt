package com.gemwallet.android.features.transfer_amount.viewmodels

import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.model.ValueFormatter
import uniffi.gemstone.GemAmountErrorDisplay
import uniffi.gemstone.GemAmountException
import uniffi.gemstone.GemValueStyle

fun GemAmountException.toAmountError(): AmountError = when (val display = display()) {
    is GemAmountErrorDisplay.None -> AmountError.None
    is GemAmountErrorDisplay.InvalidAmount -> AmountError.IncorrectAmount
    is GemAmountErrorDisplay.BelowMinimum ->
        AmountError.MinimumValue(ValueFormatter(style = GemValueStyle.AUTO).string(display.minimum, display.asset.decimals, display.asset.symbol))
    is GemAmountErrorDisplay.InsufficientBalance -> AmountError.InsufficientBalance(display.title)
}
