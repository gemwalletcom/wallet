package com.gemwallet.android.blockchain.services

import com.gemwallet.android.ext.gemChainAddresses
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentOptions
import com.wallet.core.primitives.PaymentOutcome
import com.wallet.core.primitives.PaymentQuote
import com.wallet.core.primitives.PaymentQuoteData
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemPaymentServiceInterface

class PaymentService(
    private val client: GemPaymentServiceInterface,
) {
    suspend fun getOptions(link: PaymentLink, wallet: Wallet): PaymentOptions =
        client.getOptions(link.toGem(), wallet.gemChainAddresses()).toPrimitives()

    suspend fun getQuoteData(quote: PaymentQuote, wallet: Wallet): PaymentQuoteData =
        requireNotNull(client.getQuoteData(quote.toGem(), wallet.gemChainAddresses()).toPrimitives()) {
            "Payment asks for an unsupported chain"
        }

    suspend fun confirm(quote: PaymentQuote, transactionHash: String): PaymentOutcome =
        client.confirm(quote.toGem(), transactionHash).toPrimitives()
}
