package com.gemwallet.android.blockchain.services

import com.gemwallet.android.ext.gemChainAddresses
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.PreparedPayment
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentOptions
import com.wallet.core.primitives.PaymentOutcome
import com.wallet.core.primitives.PaymentProviderName
import com.wallet.core.primitives.PaymentQuote
import com.wallet.core.primitives.PaymentQuotes
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemPaymentServiceInterface
import uniffi.gemstone.paymentProviderHasStatus

class PaymentService(
    private val client: GemPaymentServiceInterface,
) {

    fun hasStatus(provider: PaymentProviderName): Boolean = paymentProviderHasStatus(provider.toGem())

    suspend fun getPaymentOptions(link: PaymentLink, wallet: Wallet): PaymentOptions =
        client.getPaymentOptions(link.toGem(), wallet.gemChainAddresses()).toPrimitives()

    suspend fun getPreparedPayment(
        provider: PaymentProviderName,
        quotes: PaymentQuotes,
        quote: PaymentQuote,
        wallet: Wallet,
    ): PreparedPayment {
        val payment = client.getPreparedPayment(
            provider.toGem(),
            quotes.toGem(),
            quote.toGem(),
            wallet.gemChainAddresses(),
        )
        return PreparedPayment(
            quotes = payment.quotes.toPrimitives(),
            quote = payment.quote.toPrimitives(),
            actions = payment.actions,
            isRelayed = payment.isRelayed,
        )
    }

    suspend fun confirmPayment(
        provider: PaymentProviderName,
        quote: PaymentQuote,
        actionResults: List<String>,
    ): PaymentOutcome = client.confirmPayment(provider.toGem(), quote.toGem(), actionResults).toPrimitives()

    suspend fun getPaymentStatus(provider: PaymentProviderName, paymentId: String): PaymentOutcome =
        client.getPaymentStatus(provider.toGem(), paymentId).toPrimitives()
}
