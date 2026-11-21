package com.gemwallet.android.ui.components

import org.junit.Assert.assertEquals
import org.junit.Test

class TextMessageBlocksTest {
    @Test
    fun eachLineIsABlock() {
        assertEquals(listOf(""), textMessageBlocks(""))
        assertEquals(listOf("Sign in", "", "to Gem"), textMessageBlocks("Sign in\n\nto Gem"))
        assertEquals(listOf("a".repeat(1024)), textMessageBlocks("a".repeat(1024)))
    }

    @Test
    fun aLongLineBreaksAfterItsLastSpace() {
        val first = "a".repeat(1000) + " "
        val second = "b".repeat(100)

        assertEquals(listOf(first, second), textMessageBlocks(first + second))
    }

    @Test
    fun unbrokenTextIsCutEveryBlockLengthWithoutSplittingACharacter() {
        assertEquals(listOf("a".repeat(1024), "a".repeat(1024), "a"), textMessageBlocks("a".repeat(2049)))
        assertEquals(listOf("a".repeat(1023), "\uD83D\uDE00"), textMessageBlocks("a".repeat(1023) + "\uD83D\uDE00"))
    }

    @Test
    fun aMegabyteOfUnbrokenTextStaysWhole() {
        val message = "a".repeat(1024 * 1024)

        assertEquals(message, textMessageBlocks(message).joinToString(""))
    }
}
