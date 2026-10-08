package com.gemwallet.android

import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PrivacyCoverTest {

    private var gemFocused = false

    @Test
    fun leavingGemCoversAfterAMoment() = runTest {
        val cover = cover()

        cover.onFocusChanged(false)
        advanceTimeBy(100)
        assertFalse(cover.isCovered.value)

        advanceTimeBy(100)
        assertTrue(cover.isCovered.value)
    }

    @Test
    fun aGemWindowTakingFocusDoesNotCover() = runTest {
        val cover = cover()
        gemFocused = true

        cover.onFocusChanged(false)
        settle()

        assertFalse(cover.isCovered.value)
    }

    @Test
    fun regainingFocusUncoversAtOnce() = runTest {
        val cover = cover()
        cover.onFocusChanged(false)
        settle()

        cover.onFocusChanged(true)

        assertFalse(cover.isCovered.value)
    }

    @Test
    fun aGemPromptKeepsTheCoverDownUntilGemHasFocusAgain() = runTest {
        val cover = cover()

        cover.onPromptShown()
        cover.onFocusChanged(false)
        settle()
        cover.onPromptEnded()
        settle()
        assertFalse(cover.isCovered.value)

        cover.onFocusChanged(true)
        cover.onFocusChanged(false)
        settle()
        assertTrue(cover.isCovered.value)
    }

    @Test
    fun aPromptThatEndedBeforeLeavingDoesNotKeepTheCoverDown() = runTest {
        val cover = cover()

        cover.onPromptShown()
        cover.onPromptEnded()
        cover.onFocusChanged(false)
        settle()

        assertTrue(cover.isCovered.value)
    }

    @Test
    fun stoppingCoversAtOnceEvenDuringAPrompt() = runTest {
        val cover = cover()
        cover.onPromptShown()
        cover.onFocusChanged(false)

        cover.onStop()

        assertTrue(cover.isCovered.value)
    }

    @Test
    fun nothingCoversWhileTheLockIsOff() = runTest {
        val cover = cover(isLockEnabled = false)

        cover.onFocusChanged(false)
        settle()
        cover.onStop()

        assertFalse(cover.isCovered.value)
    }

    private fun TestScope.cover(isLockEnabled: Boolean = true) = PrivacyCover(backgroundScope) { gemFocused }.also { it.isLockEnabled = isLockEnabled }

    private fun TestScope.settle() {
        advanceTimeBy(1_000)
        runCurrent()
    }
}
