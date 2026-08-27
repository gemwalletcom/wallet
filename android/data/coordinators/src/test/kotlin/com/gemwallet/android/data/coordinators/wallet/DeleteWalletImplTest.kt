package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.WalletType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import uniffi.gemstone.GemWalletService
import org.junit.After
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class DeleteWalletImplTest {

    private val sessionRepository = mockk<SessionRepository>(relaxed = true)
    private val walletsRepository = mockk<WalletsRepository>(relaxed = true)
    private val deleteKeyStoreOperator = mockk<DeleteKeyStoreOperator>()

    private val walletService = mockk<GemWalletService>()

    private val delete = DeleteWalletImpl(
        sessionRepository,
        walletsRepository,
        deleteKeyStoreOperator,
        walletService,
    )

    @Before
    fun setUp() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun keepsWalletWhenKeystoreDeletionFails() = runTest {
        val wallet = mockWallet(id = "wallet-1", type = WalletType.Multicoin)
        every { walletsRepository.getWallet(wallet.id) } returns flowOf(wallet)
        every { sessionRepository.session() } returns MutableStateFlow<Session?>(null)
        every { deleteKeyStoreOperator(wallet) } returns false

        delete.deleteWallet(wallet.id, onBoard = {}, onComplete = {})

        verify { deleteKeyStoreOperator(wallet) }
        coVerify(exactly = 0) { walletService.deleteWallet(any()) }
    }

    @Test
    fun clearsWalletPreferencesWhenWalletDeleted() = runTest {
        val wallet = mockWallet(id = "wallet-1", type = WalletType.Multicoin)
        every { walletsRepository.getWallet(wallet.id) } returns flowOf(wallet)
        every { deleteKeyStoreOperator(wallet) } returns true
        coEvery { walletService.deleteWallet(any()) } returns false

        delete.deleteWallet(wallet.id, onBoard = {}, onComplete = {})

        coVerify { walletService.deleteWallet(any()) }
        coVerify { sessionRepository.reset() }
    }
}
