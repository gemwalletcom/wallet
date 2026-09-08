package com.gemwallet.android.features.transfer_amount.viewmodels

import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.model.ValueFormatter
import uniffi.gemstone.GemAmountException

fun GemAmountException.toAmountError(): AmountError = when (this) {
    is GemAmountException.InvalidNumber, is GemAmountException.PriceMissing -> AmountError.IncorrectAmount
    is GemAmountException.Zero -> AmountError.None
    is GemAmountException.BelowMinimum -> AmountError.MinimumValue(ValueFormatter(style = ValueFormatter.Style.Full).string(minimum, asset.decimals, asset.symbol))
    is GemAmountException.InsufficientBalance -> AmountError.InsufficientBalance(asset.symbol)
}
