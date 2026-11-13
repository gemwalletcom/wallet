package com.gemwallet.android.data.services.gemstone.lightning

import android.util.Log
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
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

    private fun TestScope.observer() = LightningSessionObserver(service = service, session = session, scope = this)
}
