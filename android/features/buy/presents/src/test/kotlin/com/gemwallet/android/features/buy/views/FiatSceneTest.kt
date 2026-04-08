package com.gemwallet.android.features.buy.views

import com.gemwallet.android.features.buy.viewmodels.models.FiatSuggestion
import org.junit.Assert.assertEquals
import org.junit.Test

class FiatSceneTest {

    @Test
    fun `compact width keeps only primary shortcut`() {
        val suggestions = listOf(
            FiatSuggestion.SuggestionAmount("$100", 100.0),
            FiatSuggestion.SuggestionAmount("$250", 250.0),
            FiatSuggestion.RandomAmount,
        )

        val visibleSuggestions = visibleSuggestedAmountsInAssetRow(
            suggestedAmounts = suggestions,
            isCompactWidth = true,
        )

        assertEquals(listOf("$100"), visibleSuggestions.map { it.text })
    }

    @Test
    fun `regular width keeps all shortcuts visible`() {
        val suggestions = listOf(
            FiatSuggestion.SuggestionAmount("$100", 100.0),
            FiatSuggestion.SuggestionAmount("$250", 250.0),
            FiatSuggestion.RandomAmount,
        )

        val visibleSuggestions = visibleSuggestedAmountsInAssetRow(
            suggestedAmounts = suggestions,
            isCompactWidth = false,
        )

        assertEquals(listOf("$100", "$250", "Random"), visibleSuggestions.map { it.text })
    }
}
