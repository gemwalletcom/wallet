package com.gemwallet.android.features.transactions.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockFormattedNumber
import com.gemwallet.android.testkit.mockGemHeaderAmount
import com.gemwallet.android.testkit.mockGemTransactionDetailRows
import com.gemwallet.android.testkit.mockGemTransactionFeeRow
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemNumberUnit
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
            header = GemTransactionHeader.Swap(
                from = mockGemHeaderAmount(asset = mockAsset().toGem(), amount = mockFormattedNumber(1.0, GemNumberUnit.Symbol(mockAsset().symbol)), fiat = mockFormattedNumber(2.0)),
                to = mockGemHeaderAmount(asset = mockAsset().toGem(), amount = mockFormattedNumber(2.0, GemNumberUnit.Symbol("BTC")), fiat = mockFormattedNumber(2.0)),
            ),
        )

        val head = rows.uiModel(GemTransactionDetailRow.Header, context)

        assertTrue(head is TransactionDetailsRowUIModel.SwapHead)
        assertEquals((head as TransactionDetailsRowUIModel.SwapHead).fromAsset, head.toAsset)
        assertEquals("2.00 BTC", head.toValueText)
    }

    @Test
    fun `an approval header is the asset image and a symbol header is its text`() {
        val asset = mockGemHeaderAmount(asset = mockAsset().toGem(), amount = mockFormattedNumber(1.0, GemNumberUnit.Symbol(mockAsset().symbol)), fiat = mockFormattedNumber(2.0)).asset
        val image = mockGemTransactionDetailRows(header = GemTransactionHeader.AssetImage(asset)).uiModel(GemTransactionDetailRow.Header, context)
        val symbol = mockGemTransactionDetailRows(header = GemTransactionHeader.Symbol(asset)).uiModel(GemTransactionDetailRow.Header, context)

        assertTrue("a token approval shows the asset, not its symbol as an amount", image is TransactionDetailsRowUIModel.AssetHead)
        assertTrue(symbol is TransactionDetailsRowUIModel.AmountHead)
        assertEquals(asset.symbol, (symbol as TransactionDetailsRowUIModel.AmountHead).amount)
    }

    @Test
    fun `a fee row reads the numbers core formatted`() {
        val rows = mockGemTransactionDetailRows(feeRow = mockGemTransactionFeeRow(amount = mockFormattedNumber(1.0, GemNumberUnit.Symbol("BTC"))))

        val fee = rows.uiModel(GemTransactionDetailRow.Fee, context)

        assertTrue(fee is TransactionDetailsRowUIModel.Fee)
        assertEquals(rows.feeRow.amount.text(), (fee as TransactionDetailsRowUIModel.Fee).model.subtitle)
    }
}
