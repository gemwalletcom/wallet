package com.gemwallet.android.features.activities.viewmodels.models

import android.content.Context
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockGemTransactionAmount
import com.gemwallet.android.testkit.mockGemTransactionDetailRows
import com.wallet.core.primitives.Currency
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemTransactionDetailRow
import uniffi.gemstone.GemTransactionHeader

class TransactionDetailsRowUIModelTest {

    private val context = mockk<Context> {
        every { getString(any()) } answers { "string:${firstArg<Int>()}" }
        every { getString(any(), *anyVararg()) } answers { "string:${firstArg<Int>()}" }
    }

    @Test
    fun `a swap header reads both legs from the record`() {
        val rows = mockGemTransactionDetailRows(
            header = GemTransactionHeader.Swap(from = mockGemTransactionAmount(), to = mockGemTransactionAmount()),
        )

        val head = rows.uiModel(GemTransactionDetailRow.Header, context, Currency.USD)

        assertTrue(head is TransactionDetailsRowUIModel.SwapHead)
        assertEquals((head as TransactionDetailsRowUIModel.SwapHead).fromAsset.asset, head.toAsset.asset)
    }

    @Test
    fun `an approval header is the asset image and a symbol header is its text`() {
        val asset = mockGemTransactionAmount().asset
        val image = mockGemTransactionDetailRows(header = GemTransactionHeader.AssetImage(asset)).uiModel(GemTransactionDetailRow.Header, context, Currency.USD)
        val symbol = mockGemTransactionDetailRows(header = GemTransactionHeader.Symbol(asset)).uiModel(GemTransactionDetailRow.Header, context, Currency.USD)

        assertTrue("a token approval shows the asset, not its symbol as an amount", image is TransactionDetailsRowUIModel.AssetHead)
        assertTrue(symbol is TransactionDetailsRowUIModel.AmountHead)
        assertEquals(asset.symbol, (symbol as TransactionDetailsRowUIModel.AmountHead).amount)
    }

    @Test
    fun `a fee row reads the numbers core formatted`() {
        val rows = mockGemTransactionDetailRows()

        val fee = rows.uiModel(GemTransactionDetailRow.Fee, context, Currency.USD)

        assertTrue(fee is TransactionDetailsRowUIModel.Fee)
        assertEquals(rows.feeRow.amount.text(), (fee as TransactionDetailsRowUIModel.Fee).model.subtitle)
    }
}
