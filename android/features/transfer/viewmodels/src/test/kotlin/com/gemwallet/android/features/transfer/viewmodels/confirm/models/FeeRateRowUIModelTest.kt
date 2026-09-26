package com.gemwallet.android.features.transfer.viewmodels.confirm.models

import android.content.Context
import com.gemwallet.android.domains.confirm.FeeRateUIModel
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemFormattedNumber
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import io.mockk.every
import io.mockk.mockk
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.FeeUnitType
import uniffi.gemstone.GemFeeRateRow
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.feeAmount
import java.math.BigInteger
import java.util.Locale

class FeeRateRowUIModelTest {

    private val context = mockk<Context> {
        every { getString(any()) } returns "text"
    }

    private val defaultLocale = Locale.getDefault()

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    @After
    fun tearDown() {
        Locale.setDefault(defaultLocale)
    }

    @Test
    fun aPriorityRowCarriesTheTitleRateFiatAndEmoji() {
        val model = FeeRateUIModel(
            row = GemFeeRateRow(
                priority = FeePriority.Fast.toGem(),
                fee = BigInteger("500000000000000000"),
                amount = feeAmount(mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18).toGem(), BigInteger("500000000000000000"), 1.0, Currency.USD.toGem()),
                value = GemLocalizedText.FeeRate(mockGemFormattedNumber(value = 2.5, unit = GemNumberUnit.Plain, display = GemNumberDisplay.Number(precision = GemPrecision.Fraction(min = 2u, max = 2u))), FeeUnitType.GWEI),
                isSelected = true,
            ),
        ).rowUIModel(context)

        assertEquals(FeePriority.Fast, model.priority)
        assertEquals(true, model.isSelected)
        assertEquals("text", model.model.title)
        assertEquals("2.50 text", model.model.subtitle)
        assertEquals("$0.5", model.model.subtitleExtra)
        assertEquals("⚡️", model.emoji)
    }

    @Test
    fun theCustomRowHasNoPriorityAndShowsOnlyTheSelectedRate() {
        val empty = customFeeRowUIModel(context, null, null)
        assertNull(empty.priority)
        assertEquals("text", empty.model.title)
        assertNull(empty.model.subtitle)
        assertNull(empty.model.subtitleExtra)

        val selected = customFeeRowUIModel(
            context,
            GemLocalizedText.FeeRate(mockGemFormattedNumber(value = 4.0, unit = GemNumberUnit.Plain, display = GemNumberDisplay.Number(precision = GemPrecision.Fraction(min = 2u, max = 2u))), FeeUnitType.NATIVE),
            "$1.2",
        )
        assertEquals("4.00", selected.model.subtitle)
        assertEquals("$1.2", selected.model.subtitleExtra)
    }
}
