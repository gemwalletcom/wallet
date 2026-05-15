package com.gemwallet.android.features.import_wallet.components

import androidx.compose.ui.graphics.Color
import com.wallet.core.primitives.WalletType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ImportInputTest {

    @Test
    fun phraseImport_showsPhraseSuggestions() {
        assertTrue(supportsPhraseSuggestions(WalletType.Single))
    }

    @Test
    fun privateKeyImport_hidesPhraseSuggestions() {
        assertFalse(supportsPhraseSuggestions(WalletType.PrivateKey))
    }

    @Test
    fun invalidPhraseWords_highlightsOnlyNonBlankWordRanges() {
        val result = highlightInvalidPhraseWords(
            text = "legal  nope tail",
            errorColor = Color.Red,
            invalidWords = setOf("", "nope"),
        )

        assertEquals("legal  nope tail", result.text)
        assertEquals(1, result.spanStyles.size)
        assertEquals(7, result.spanStyles.single().start)
        assertEquals(11, result.spanStyles.single().end)
        assertTrue(result.spanStyles.all { it.start < it.end && it.end <= result.length })
    }
}
