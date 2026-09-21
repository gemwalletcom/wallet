package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.confirm.FeeRateUIModel
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockAssetPriceValue
import com.gemwallet.android.testkit.mockFormattedNumber
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
import uniffi.gemstone.GemNumberUnit
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
                value = GemLocalizedText.FeeRate(mockFormattedNumber(value = 2.5, unit = GemNumberUnit.Plain), FeeUnitType.GWEI),
            ),
            feeAsset = mockAssetPriceValue(mockAssetEthereum(), mockAssetPriceInfo(price = 1.0)),
        ).rowUIModel(context)

        assertEquals(FeePriority.Fast, model.priority)
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
            GemLocalizedText.FeeRate(mockFormattedNumber(value = 4.0, unit = GemNumberUnit.Plain), FeeUnitType.NATIVE),
            "$1.2",
        )
        assertEquals("4.00", selected.model.subtitle)
        assertEquals("$1.2", selected.model.subtitleExtra)
    }
}
