package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.data.adapters.config.UserConfig
import com.gemwallet.android.application.session.cases.ClearSession
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockWalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
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

    private val clearSession = mockk<ClearSession>(relaxed = true)
    private val deleteKeyStoreOperator = mockk<DeleteKeyStoreOperator>()

    private val walletService = mockk<GemWalletService>()

    private val userConfig = mockk<UserConfig>(relaxed = true)

    private val delete = DeleteWalletImpl(
        clearSession,
        deleteKeyStoreOperator,
        walletService,
        userConfig,
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
        val walletId = mockWalletId()
        every { deleteKeyStoreOperator(walletId) } returns false

        delete.deleteWallet(walletId, onBoard = {}, onComplete = {})

        verify { deleteKeyStoreOperator(walletId) }
        coVerify(exactly = 0) { walletService.deleteWallet(any()) }
    }

    @Test
    fun resetsSessionWhenLastWalletDeleted() = runTest {
        val walletId = mockWalletId()
        every { deleteKeyStoreOperator(walletId) } returns true
        coEvery { walletService.deleteWallet(walletId.id) } returns false

        delete.deleteWallet(walletId, onBoard = {}, onComplete = {})

        coVerify { walletService.deleteWallet(walletId.id) }
        coVerify { clearSession.clearSession() }
        verify { userConfig.reload() }
    }
}
