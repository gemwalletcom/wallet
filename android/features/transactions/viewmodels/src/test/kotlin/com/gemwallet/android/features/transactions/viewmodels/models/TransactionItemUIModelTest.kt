package com.gemwallet.android.features.transactions.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockGemAssetIcon
import com.gemwallet.android.testkit.mockGemFormattedNumber
import com.gemwallet.android.testkit.mockGemHeaderAmount
import com.gemwallet.android.testkit.mockGemTransactionDetailRows
import com.gemwallet.android.testkit.mockGemTransactionFeeRow
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.GemTransactionDetailRow
import uniffi.gemstone.GemTransactionHeader

class TransactionItemUIModelTest {

    private val context = mockk<Context> {
        every { getString(any()) } answers { "string:${firstArg<Int>()}" }
        every { getString(any(), *anyVararg()) } answers { "string:${firstArg<Int>()}" }
    }

    @Test
    fun `a swap header reads both legs from the record`() {
        val rows = mockGemTransactionDetailRows(
            header = GemTransactionHeader.Swap(
                from = mockGemHeaderAmount(asset = mockAsset().toGem(), amount = mockGemFormattedNumber(value = 1.0, unit = GemNumberUnit.Symbol(mockAsset().symbol)), fiat = mockGemFormattedNumber(value = 2.0)),
                to = mockGemHeaderAmount(
                    asset = mockAsset().toGem(),
                    amount = mockGemFormattedNumber(value = 2.0, unit = GemNumberUnit.Symbol("BTC"), display = GemNumberDisplay.Number(precision = GemPrecision.Fraction(min = 2u, max = 2u))),
                    fiat = mockGemFormattedNumber(value = 2.0),
                ),
            ),
        )

        val head = rows.uiModel(GemTransactionDetailRow.Header, context)

        assertTrue(head is TransactionItemUIModel.SwapHead)
        assertEquals((head as TransactionItemUIModel.SwapHead).fromAsset, head.toAsset)
        assertEquals("2.00 BTC", head.toValueText)
    }

    @Test
    fun `an approval header is the asset image and a symbol header is its text`() {
        val asset = mockGemHeaderAmount(asset = mockAsset().toGem(), amount = mockGemFormattedNumber(value = 1.0, unit = GemNumberUnit.Symbol(mockAsset().symbol)), fiat = mockGemFormattedNumber(value = 2.0)).asset
        val image = mockGemTransactionDetailRows(header = GemTransactionHeader.AssetImage(mockGemAssetIcon())).uiModel(GemTransactionDetailRow.Header, context)
        val symbol = mockGemTransactionDetailRows(header = GemTransactionHeader.Symbol(asset, mockGemAssetIcon())).uiModel(GemTransactionDetailRow.Header, context)

        assertTrue("a token approval shows the asset, not its symbol as an amount", image is TransactionItemUIModel.AssetHead)
        assertTrue(symbol is TransactionItemUIModel.AmountHead)
        assertEquals(asset.symbol, (symbol as TransactionItemUIModel.AmountHead).amount)
    }

    @Test
    fun `a fee row reads the numbers core formatted`() {
        val rows = mockGemTransactionDetailRows(feeRow = mockGemTransactionFeeRow(amount = mockGemFormattedNumber(value = 1.0, unit = GemNumberUnit.Symbol("BTC"))))

        val fee = rows.uiModel(GemTransactionDetailRow.Fee, context)

        assertTrue(fee is TransactionItemUIModel.Fee)
        assertEquals(rows.feeRow.amount.text(), (fee as TransactionItemUIModel.Fee).model.subtitle)
    }
}
