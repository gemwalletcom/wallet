package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.WalletPreferences
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
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
import org.junit.After
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class DeleteWalletImplTest {

    private val sessionRepository = mockk<SessionRepository>(relaxed = true)
    private val walletsRepository = mockk<WalletsRepository>(relaxed = true)
    private val deleteKeyStoreOperator = mockk<DeleteKeyStoreOperator>()
    private val walletPreferences = mockk<WalletPreferences>(relaxed = true)
    private val walletPreferencesFactory = mockk<WalletPreferencesFactory> {
        every { create(any()) } returns walletPreferences
    }

    private val delete = DeleteWalletImpl(
        sessionRepository,
        walletsRepository,
        deleteKeyStoreOperator,
        walletPreferencesFactory,
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
        coVerify(exactly = 0) { walletsRepository.removeWallet(any()) }
        verify(exactly = 0) { walletPreferences.clear() }
    }

    @Test
    fun clearsWalletPreferencesWhenWalletDeleted() = runTest {
        val wallet = mockWallet(id = "wallet-1", type = WalletType.Multicoin)
        every { walletsRepository.getWallet(wallet.id) } returns flowOf(wallet)
        every { walletsRepository.getAll() } returns flowOf(emptyList())
        every { sessionRepository.session() } returns MutableStateFlow<Session?>(null)
        every { deleteKeyStoreOperator(wallet) } returns true
        coEvery { walletsRepository.removeWallet(wallet.id) } returns true

        delete.deleteWallet(wallet.id, onBoard = {}, onComplete = {})

        coVerify { walletsRepository.removeWallet(wallet.id) }
        verify { walletPreferencesFactory.create(wallet.id.id) }
        verify { walletPreferences.clear() }
    }
}
