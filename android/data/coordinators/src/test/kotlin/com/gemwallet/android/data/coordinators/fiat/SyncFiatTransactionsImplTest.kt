package com.gemwallet.android.data.coordinators.fiat

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemFiatService

class SyncFiatTransactionsImplTest {

    private val walletId = WalletId("wallet-1")
    private val getSession = mockk<GetSession>()
    private val fiatService = mockk<GemFiatService> {
        coEvery { syncTransactions(any()) } returns Unit
    }
    private val subject = SyncFiatTransactionsImpl(getSession, fiatService)

    @Test
    fun syncFiatTransactions_withoutSession_skipsWork() = runTest {
        every { getSession() } returns MutableStateFlow(null)

        subject()

        coVerify(exactly = 0) { fiatService.syncTransactions(any()) }
    }

    @Test
    fun syncFiatTransactions_usesCurrentSessionWallet() = runTest {
        every { getSession() } returns MutableStateFlow(mockSession(wallet = mockWallet(id = walletId.id)))

        subject()

        coVerify { fiatService.syncTransactions(walletId.id) }
    }
}
