package com.gemwallet.android.features.swap.viewmodels.localization

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.ext.boldMarkdown
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemSwapButtonAction
import uniffi.gemstone.GemSwapErrorDisplay
import uniffi.gemstone.GemValueStyle

@StringRes
internal fun GemSwapButtonAction.stringRes(): Int = when (this) {
    GemSwapButtonAction.InsufficientBalance -> R.string.transfer_insufficient_balance
    is GemSwapButtonAction.UseMinimumAmount -> R.string.swap_use_minimum_amount
    GemSwapButtonAction.RetryQuote,
    GemSwapButtonAction.RetryTransfer -> R.string.common_try_again
    GemSwapButtonAction.Swap -> R.string.wallet_swap
}

internal fun GemSwapErrorDisplay.text(context: Context): String = when (this) {
    is GemSwapErrorDisplay.NotSupportedAsset -> context.getString(R.string.errors_swap_not_supported_asset)
    is GemSwapErrorDisplay.NoQuote -> context.getString(R.string.errors_swap_no_quote_available)
    is GemSwapErrorDisplay.MinimumAmount -> context.getString(
        R.string.errors_swap_minimum_amount,
        ValueFormatter(style = GemValueStyle.AUTO).string(minAmount, asset.toPrimitives()).boldMarkdown(),
    )
    is GemSwapErrorDisplay.AmountTooSmall -> context.getString(R.string.errors_swap_amount_too_small)
}
