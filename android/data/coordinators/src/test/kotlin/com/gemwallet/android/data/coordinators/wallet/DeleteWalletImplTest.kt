package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.testkit.mockWalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemWalletDeletion
import uniffi.gemstone.GemWalletService

@OptIn(ExperimentalCoroutinesApi::class)
class DeleteWalletImplTest {

    private val walletService = mockk<GemWalletService>()

    private val userConfig = mockk<UserConfig>(relaxed = true)

    private val delete = DeleteWalletImpl(walletService, userConfig)

    @Before
    fun setUp() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun keepsTheUserInPlaceWhenTheDeleteFails() = runTest {
        val walletId = mockWalletId()
        var onBoarded = false
        var completed = false
        coEvery { walletService.deleteWallet(walletId.id) } throws IllegalStateException("keystore delete failed")

        delete.deleteWallet(walletId, onBoard = { onBoarded = true }, onComplete = { completed = true })

        assertFalse(onBoarded)
        assertFalse(completed)
        verify(exactly = 0) { userConfig.reload() }
    }

    @Test
    fun onboardsOnlyWhenTheLastWalletIsDeleted() = runTest {
        val walletId = mockWalletId()
        var onBoarded = false
        var completed = false
        coEvery { walletService.deleteWallet(walletId.id) } returns GemWalletDeletion.WALLETS_REMAINING

        delete.deleteWallet(walletId, onBoard = { onBoarded = true }, onComplete = { completed = true })

        assertFalse(onBoarded)
        assertTrue(completed)
        verify(exactly = 0) { userConfig.reload() }

        coEvery { walletService.deleteWallet(walletId.id) } returns GemWalletDeletion.LAST_WALLET_DELETED

        delete.deleteWallet(walletId, onBoard = { onBoarded = true }, onComplete = { completed = true })

        assertTrue(onBoarded)
        coVerify { walletService.deleteWallet(walletId.id) }
        verify { userConfig.reload() }
    }
}
