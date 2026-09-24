package com.gemwallet.android.features.import_wallet.viewmodels

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class PhraseInputTest {

    @Test
    fun `the word at the cursor is the one being edited`() {
        val input = PhraseInput.of("abandon woo zoo", 11)

        assertEquals("woo", input.word)
        assertEquals(3, input.wordCursor)
        assertTrue(PhraseInput.of("abandon ", 8).word.isEmpty())
        assertEquals(3, PhraseInput.of("woo", null).wordCursor)
        assertEquals(3, PhraseInput.of("woo", 99).cursor)
    }

    @Test
    fun `completing replaces the word and moves past the separator`() {
        assertEquals(PhraseInput("abandon wood ", 13), PhraseInput.of("abandon woo", null).completing("wood"))
        assertEquals(PhraseInput("wood zoo", 5), PhraseInput.of("woo zoo", 3).completing("wood"))
        assertEquals(PhraseInput("abandon wool zoo", 13), PhraseInput.of("abandon wo zoo", 9).completing("wool"))
        assertEquals(PhraseInput("wood ", 5), PhraseInput.of("wo", 1).completing("wood"))
    }

    @Test
    fun `the cursor counts utf16 units`() {
        val input = PhraseInput.of("🙂 woo", 6)

        assertEquals("woo", input.word)
        assertEquals(PhraseInput("🙂 wood ", 8), input.completing("wood"))
    }
}
