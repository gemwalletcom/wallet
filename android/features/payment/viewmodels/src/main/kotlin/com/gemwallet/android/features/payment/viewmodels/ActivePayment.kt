package com.gemwallet.android.features.payment.viewmodels

import com.gemwallet.android.model.PreparedPayment
import com.wallet.core.primitives.PaymentProviderName
import com.wallet.core.primitives.PaymentQuote
import com.wallet.core.primitives.PaymentQuotes
import com.wallet.core.primitives.TransactionPaymentMetadata
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.PaymentAction

internal data class ActivePayment(
    val provider: PaymentProviderName,
    val quotes: PaymentQuotes,
    val wallet: Wallet,
    val quote: PaymentQuote? = null,
    val collecting: PaymentQuote? = null,
    val actions: List<PaymentAction> = emptyList(),
    val results: List<String> = emptyList(),
    val completed: Int = 0,
    val isRelayed: Boolean = false,
) {
    val step: Step?
        get() = actions.getOrNull(completed)?.let { Step(it, completed) }

    fun paymentMetadata(quote: PaymentQuote) = TransactionPaymentMetadata(
        paymentId = quote.paymentId,
        merchant = quotes.merchant,
        provider = provider,
    )

    fun collecting(quote: PaymentQuote) = copy(collecting = quote)

    fun prepared(payment: PreparedPayment) = copy(
        quote = payment.quote,
        collecting = null,
        actions = payment.actions,
        results = List(payment.actions.size) { "" },
        completed = 0,
        isRelayed = payment.isRelayed,
    )

    fun completing(result: String): ActivePayment {
        val index = completed.takeIf { it in actions.indices } ?: return this
        return copy(
            results = results.toMutableList().also { it[index] = result },
            completed = completed + 1,
        )
    }

    data class Step(val action: PaymentAction, val index: Int)
}
