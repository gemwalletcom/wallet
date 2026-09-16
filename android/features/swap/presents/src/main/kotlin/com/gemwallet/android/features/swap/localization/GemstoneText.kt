package com.gemwallet.android.features.swap.localization

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.boldMarkdown
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemSwapErrorDisplay
import uniffi.gemstone.GemValueStyle

@Composable
internal fun GemSwapErrorDisplay.text(): String = when (this) {
    is GemSwapErrorDisplay.NotSupportedAsset -> stringResource(R.string.errors_swap_not_supported_asset)
    is GemSwapErrorDisplay.NoQuote -> stringResource(R.string.errors_swap_no_quote_available)
    is GemSwapErrorDisplay.MinimumAmount -> stringResource(
        R.string.errors_swap_minimum_amount,
        ValueFormatter(style = GemValueStyle.AUTO).string(minAmount, asset.toPrimitives()).boldMarkdown(),
    )
    is GemSwapErrorDisplay.AmountTooSmall -> stringResource(R.string.errors_swap_amount_too_small)
}
