package com.gemwallet.android.data.services.gemstone.lightning

import android.util.Log
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemLightningServiceInterface
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletSessionServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class LightningSessionObserverTest {

    private val walletId = "multicoin_0xabc"
    private val service = mockk<GemLightningServiceInterface>(relaxed = true)
    private val session = mockk<GemWalletSessionServiceInterface> {
        every { getCurrentWalletId() } returns walletId
    }

    @Before
    fun setUp() {
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
    }

    @After
    fun tearDown() {
        unmockkStatic(Log::class)
    }

    @Test
    fun `no current wallet opens nothing`() = runTest {
        every { session.getCurrentWalletId() } returns null

        observer().openLightning()
        advanceUntilIdle()

        coVerify(exactly = 0) { service.openEnabled(any()) }
    }

    @Test
    fun `a cancelled open is not logged`() = runTest {
        coEvery { service.openEnabled(walletId) } throws GemServiceException.Cancelled()

        observer().openLightning()
        advanceUntilIdle()

        verify(exactly = 0) { Log.e(any(), any(), any()) }
    }

    @Test
    fun `any other open failure is logged`() = runTest {
        coEvery { service.openEnabled(walletId) } throws GemServiceException.Core("storage")

        observer().openLightning()
        advanceUntilIdle()

        verify(exactly = 1) { Log.e(any(), any(), any()) }
    }

    @Test
    fun `an open already running for the wallet is not repeated`() = runTest {
        val opened = CompletableDeferred<Boolean>()
        coEvery { service.openEnabled(walletId) } coAnswers { opened.await() }
        val subject = observer()

        subject.openLightning()
        subject.openLightning()
        advanceUntilIdle()
        opened.complete(true)
        advanceUntilIdle()

        coVerify(exactly = 1) { service.openEnabled(walletId) }
    }

    @Test
    fun `an open requested after locking is not deduplicated against the discarded one`() = runTest {
        val discarded = CompletableDeferred<Boolean>()
        coEvery { service.openEnabled(walletId) } coAnswers { discarded.await() } coAndThen { true }
        val subject = observer()

        subject.openLightning()
        advanceUntilIdle()
        subject.lock()
        subject.openLightning()
        advanceUntilIdle()
        discarded.complete(false)
        advanceUntilIdle()

        coVerify(exactly = 2) { service.openEnabled(walletId) }
        verify(exactly = 1) { service.lockAll() }
    }

    @Test
    fun `the discarded open finishing does not release the open started after locking`() = runTest {
        val discarded = CompletableDeferred<Boolean>()
        val reopened = CompletableDeferred<Boolean>()
        coEvery { service.openEnabled(walletId) } coAnswers { discarded.await() } coAndThen { reopened.await() }
        val subject = observer()

        subject.openLightning()
        advanceUntilIdle()
        subject.lock()
        subject.openLightning()
        advanceUntilIdle()
        discarded.complete(false)
        advanceUntilIdle()
        subject.openLightning()
        advanceUntilIdle()
        reopened.complete(true)
        advanceUntilIdle()

        coVerify(exactly = 2) { service.openEnabled(walletId) }
    }

    private fun TestScope.observer() = LightningSessionObserver(service = service, session = session, scope = this)
}
