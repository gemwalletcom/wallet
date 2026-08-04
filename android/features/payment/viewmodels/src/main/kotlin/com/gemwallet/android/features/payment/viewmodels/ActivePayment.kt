package com.gemwallet.android.features.payment.viewmodels

import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemPaymentProviderName
import uniffi.gemstone.GemPaymentQuote
import uniffi.gemstone.GemPaymentQuotes
import uniffi.gemstone.PaymentAction

internal data class ActivePayment(
    val provider: GemPaymentProviderName,
    val quotes: GemPaymentQuotes,
    val wallet: Wallet,
    val quote: GemPaymentQuote? = null,
    val collecting: GemPaymentQuote? = null,
    val actions: List<PaymentAction> = emptyList(),
    val order: List<Int> = emptyList(),
    val results: List<String> = emptyList(),
    val completed: Int = 0,
) {
    val step: Step?
        get() = order.getOrNull(completed)?.let { Step(actions[it], it) }

    fun collecting(quote: GemPaymentQuote) = copy(collecting = quote)

    fun prepared(quote: GemPaymentQuote, actions: List<PaymentAction>) = copy(
        quote = quote,
        collecting = null,
        actions = actions,
        order = signaturesFirst(actions),
        results = List(actions.size) { "" },
        completed = 0,
    )

    fun completing(result: String): ActivePayment {
        val index = order.getOrNull(completed) ?: return this
        return copy(
            results = results.toMutableList().also { it[index] = result },
            completed = completed + 1,
        )
    }

    val isRelayed: Boolean
        get() = actions.none { it is PaymentAction.SendTransaction }

    data class Step(val action: PaymentAction, val index: Int)
}

private fun signaturesFirst(actions: List<PaymentAction>): List<Int> {
    val signatures = actions.indices.filter {
        actions[it] is PaymentAction.SignMessage || actions[it] is PaymentAction.SignTransaction
    }
    return signatures + actions.indices.filter { it !in signatures }
}
