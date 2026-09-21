package com.gemwallet.android.features.buy.localization

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.FiatQuoteType
import uniffi.gemstone.GemFiatAmountCheck
import uniffi.gemstone.GemFiatButtonAction
import uniffi.gemstone.GemFiatQuotePhase
import uniffi.gemstone.GemFiatQuotesMessage
import uniffi.gemstone.GemFiatViewState

@StringRes
fun FiatQuoteType.titleRes(): Int = when (this) {
    FiatQuoteType.Buy -> R.string.buy_title
    FiatQuoteType.Sell -> R.string.sell_title
}

@StringRes
fun FiatQuoteType.actionRes(): Int = when (this) {
    FiatQuoteType.Buy -> R.string.wallet_buy
    FiatQuoteType.Sell -> R.string.wallet_sell
}

@StringRes
fun GemFiatButtonAction.stringRes(): Int = when (this) {
    GemFiatButtonAction.CONTINUE -> R.string.common_continue
    GemFiatButtonAction.RETRY_QUOTE -> R.string.common_try_again
}

fun GemFiatAmountCheck.string(context: Context): String? = when (this) {
    is GemFiatAmountCheck.BelowMinimum -> context.getString(R.string.transfer_minimum_amount, minimum.text())
    is GemFiatAmountCheck.AboveMaximum -> context.getString(R.string.transfer_maximum_amount, maximum.text())
    is GemFiatAmountCheck.InsufficientBalance -> context.getString(R.string.transfer_insufficient_balance, title)
    GemFiatAmountCheck.Valid -> null
}

fun GemFiatViewState.amountErrorText(context: Context): String? = when (val phase = phase) {
    GemFiatQuotePhase.InvalidInput -> context.getString(R.string.errors_invalid_amount)

    is GemFiatQuotePhase.Invalid -> phase.check.string(context)

    GemFiatQuotePhase.Ready -> amountCheck.string(context)

    GemFiatQuotePhase.NoInput,
    is GemFiatQuotePhase.Loading,
    GemFiatQuotePhase.NoQuotes,
    is GemFiatQuotePhase.Failed,
    -> null
}

fun GemFiatViewState.quotesMessage(context: Context): String? = when (val message = quotesMessage()) {
    GemFiatQuotesMessage.EnterAmount -> context.getString(
        R.string.input_enter_amount_to,
        context.getString(quoteType.toPrimitives().actionRes()),
    )

    GemFiatQuotesMessage.NoResults -> context.getString(R.string.buy_no_results)

    is GemFiatQuotesMessage.Failed -> message.error.text(context)

    null -> null
}
