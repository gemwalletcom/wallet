package com.gemwallet.android.features.fiat_connect.presents

import com.gemwallet.android.testkit.mockFormattedNumber
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemFiatSuggestedAmount

class FiatSceneTest {

    private val suggestions = listOf(
        GemFiatSuggestedAmount(100u, mockFormattedNumber(100.0)),
        GemFiatSuggestedAmount(250u, mockFormattedNumber(250.0)),
    )

    @Test
    fun `compact width keeps only primary shortcut`() {
        val visibleSuggestions = visibleSuggestedAmountsInAssetRow(
            suggestedAmounts = suggestions,
            isCompactWidth = true,
        )

        assertEquals(listOf(100u), visibleSuggestions.map { it.amount })
    }

    @Test
    fun `regular width keeps all shortcuts visible`() {
        val visibleSuggestions = visibleSuggestedAmountsInAssetRow(
            suggestedAmounts = suggestions,
            isCompactWidth = false,
        )

        assertEquals(listOf(100u, 250u), visibleSuggestions.map { it.amount })
    }
}
