package com.gemwallet.android.features.wallets.viewmodels

import com.gemwallet.android.application.wallet.cases.DeleteWallet
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class WalletsViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val service = mockk<GemWalletServiceInterface>(relaxed = true) {
        every { walletsLimit() } returns 100u
    }
    private val getAllWallets = mockk<GetAllWallets> {
        every { getAllWallets() } returns flowOf(emptyList())
    }

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun `adding a wallet follows core's limit answer`() = runTest(testDispatcher) {
        val viewModel = WalletsViewModel(getAllWallets, mockk<SetCurrentWallet>(), service, mockk<DeleteWallet>())
        var opened = 0

        every { service.canAddWallet() } returns false
        viewModel.onAddWallet { opened += 1 }
        assertTrue(viewModel.isWalletsLimitReached.first { it })
        assertEquals(0, opened)

        viewModel.dismissWalletsLimit()
        every { service.canAddWallet() } returns true
        val allowed = CompletableDeferred<Unit>()
        viewModel.onAddWallet {
            opened += 1
            allowed.complete(Unit)
        }
        allowed.await()
        assertEquals(1 to false, opened to viewModel.isWalletsLimitReached.value)
    }
}
