package com.gemwallet.android.data.coordinators.support

import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemSupportServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class SupportMessagesCoordinatorTest {

    @Test
    fun `a message keeps sending after the chat that sent it is gone`() = runTest {
        val delivered = CompletableDeferred<Unit>()
        var finished = false
        val supportService = mockk<GemSupportServiceInterface> {
            coEvery { sendText(any()) } coAnswers {
                delivered.await()
                finished = true
            }
        }
        val coordinator = SupportMessagesCoordinator(supportService, TestScope(testScheduler))

        val chat = launch(start = CoroutineStart.UNDISPATCHED) { coordinator.sendText("My swap is stuck") }
        chat.cancel()
        delivered.complete(Unit)
        advanceUntilIdle()

        assertEquals(true, finished)
    }

    @Test
    fun `every picked photo is sent even when one fails`() = runTest {
        val first = byteArrayOf(1)
        val second = byteArrayOf(2)
        val supportService = mockk<GemSupportServiceInterface> {
            coEvery { sendImage(first) } throws GemServiceException.Gateway("offline")
            coEvery { sendImage(second) } returns Unit
        }
        val coordinator = SupportMessagesCoordinator(supportService, TestScope(testScheduler))

        val failure = runCatching { coordinator.sendImages(listOf(first, second)) }.exceptionOrNull()

        coVerify(exactly = 1) { supportService.sendImage(second) }
        assertEquals("offline", (failure as GemServiceException.Gateway).msg)
    }
}
