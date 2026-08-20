package com.gemwallet.android.features.payment.viewmodels

import com.gemwallet.android.features.payment.viewmodels.model.toUIModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentQuote
import org.junit.Assert.assertEquals
import org.junit.Test

class PaymentQuoteUIModelTest {

    @Test
    fun quoteAmountReadsTheChainCoin() {
        val quote = PaymentQuote(
            id = "option_1",
            link = PaymentLink.WalletConnectPay("pay_1"),
            assetId = AssetId(Chain.Ethereum),
            value = "14192816625800",
            expiresAt = null,
            collectDataUrl = null,
            providerData = "{}",
        )

        val model = quote.toUIModel()

        assertEquals("Ethereum", model.name)
        assertEquals("ETH", model.symbol)
        assertEquals("<0.0001 ETH", model.amountText)
    }
}
