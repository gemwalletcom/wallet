package com.gemwallet.android.features.transactions.viewmodels.models

import android.content.Context
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockGemAssetIcon
import com.gemwallet.android.testkit.mockGemFeeAmount
import com.gemwallet.android.testkit.mockGemFeeText
import com.gemwallet.android.testkit.mockGemFormattedNumber
import com.gemwallet.android.testkit.mockGemTransactionDetailRows
import com.gemwallet.android.testkit.mockGemTransactionFeeRow
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
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
    fun `a fee row shows the value core picked and its details show the amount with its fiat value`() {
        val amount = mockGemFormattedNumber(value = 1.0, unit = GemNumberUnit.Symbol("BTC"))
        val fiat = mockGemFormattedNumber(value = 2.0, unit = GemNumberUnit.Currency("USD"))
        val rows = mockGemTransactionDetailRows(feeRow = mockGemTransactionFeeRow(text = mockGemFeeText(value = fiat), fee = mockGemFeeAmount(amount = amount, fiat = fiat)))

        val fee = rows.uiModel(GemTransactionDetailRow.Fee, context) as TransactionItemUIModel.Fee

        assertEquals(fiat.text() to null, fee.model.subtitle to fee.model.subtitleExtra)
        assertEquals(amount.text() to fiat.text(), fee.details.subtitle to fee.details.subtitleExtra)
    }
}
