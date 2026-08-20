package com.gemwallet.android.features.payment.viewmodels

import com.gemwallet.android.model.PaymentData
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentQuote
import com.wallet.core.primitives.PaymentQuotes
import com.wallet.core.primitives.Wallet

internal data class ActivePayment(
    val link: PaymentLink,
    val quotes: PaymentQuotes,
    val wallet: Wallet,
    val quote: PaymentQuote? = null,
    val collecting: PaymentQuote? = null,
) {
    fun paymentData(quote: PaymentQuote) = PaymentData(
        quote = quote,
        merchant = quotes.merchant,
    )

    fun collecting(quote: PaymentQuote) = copy(collecting = quote)

    fun prepared(quote: PaymentQuote) = copy(quote = quote, collecting = null)
}
