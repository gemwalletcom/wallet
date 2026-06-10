package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.blockchain.operators.DeleteKeyStoreOperator
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.WalletType
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Test

class DeleteWalletImplTest {

    private val sessionRepository = mockk<SessionRepository>(relaxed = true)
    private val walletsRepository = mockk<WalletsRepository>(relaxed = true)
    private val deleteKeyStoreOperator = mockk<DeleteKeyStoreOperator>()
    private val syncSubscription = mockk<SyncSubscription>(relaxed = true)

    private val delete = DeleteWalletImpl(sessionRepository, walletsRepository, deleteKeyStoreOperator, syncSubscription)

    @Test
    fun keepsWalletWhenKeystoreDeletionFails() = runTest {
        val wallet = mockWallet(id = "wallet-1", type = WalletType.Multicoin)
        every { walletsRepository.getWallet(wallet.id) } returns flowOf(wallet)
        every { sessionRepository.session() } returns MutableStateFlow<Session?>(null)
        every { deleteKeyStoreOperator(wallet) } returns false

        delete.deleteWallet(wallet.id, onBoard = {}, onComplete = {})

        verify { deleteKeyStoreOperator(wallet) }
        coVerify(exactly = 0) { walletsRepository.removeWallet(any()) }
    }
}
