package com.gemwallet.android.features.transactions.viewmodels.models

import android.content.Context
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockGemAssetIcon
import com.gemwallet.android.testkit.mockGemFormattedNumber
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

class TransactionItemUIModelTest {

    private val context = mockk<Context> {
        every { getString(any()) } answers { "string:${firstArg<Int>()}" }
        every { getString(any(), *anyVararg()) } answers { "string:${firstArg<Int>()}" }
    }

    @Test
    fun `the header row draws the header Core built`() {
        val rows = mockGemTransactionDetailRows(header = GemTransactionHeader.AssetImage(mockGemAssetIcon()))

        assertEquals(TransactionItemUIModel.Head(rows.header), rows.uiModel(GemTransactionDetailRow.Header, context))
    }

    @Test
    fun `a fee row reads the numbers core formatted`() {
        val rows = mockGemTransactionDetailRows(feeRow = mockGemTransactionFeeRow(amount = mockGemFormattedNumber(value = 1.0, unit = GemNumberUnit.Symbol("BTC"))))

        val fee = rows.uiModel(GemTransactionDetailRow.Fee, context)

        assertTrue(fee is TransactionItemUIModel.Fee)
        assertEquals(rows.feeRow.amount.text(), (fee as TransactionItemUIModel.Fee).model.subtitle)
    }
}
