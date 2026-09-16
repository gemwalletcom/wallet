package com.gemwallet.android.features.swap.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemSwapButtonAction

@StringRes
internal fun GemSwapButtonAction.stringRes(): Int = when (this) {
    GemSwapButtonAction.InsufficientBalance -> R.string.transfer_insufficient_balance
    is GemSwapButtonAction.UseMinimumAmount -> R.string.swap_use_minimum_amount
    GemSwapButtonAction.RetryQuote,
    GemSwapButtonAction.RetryTransfer -> R.string.common_try_again
    GemSwapButtonAction.Swap -> R.string.wallet_swap
}
