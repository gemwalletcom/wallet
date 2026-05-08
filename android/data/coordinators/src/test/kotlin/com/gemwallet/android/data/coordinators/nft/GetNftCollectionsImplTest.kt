package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.cases.nft.GetListNftCase
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Test

class GetNftCollectionsImplTest {

    private val getListNftCase = mockk<GetListNftCase>()
    private val sessionRepository = mockk<SessionRepository>()

    @Test
    fun rescopesCollectionsWhenWalletChanges() = runTest {
        val sessionA = mockSession(wallet = mockWallet(id = "wallet-a"))
        val sessionB = mockSession(wallet = mockWallet(id = "wallet-b"))
        val sessions = MutableStateFlow<Session?>(sessionA)
        every { sessionRepository.session() } returns sessions
        every { getListNftCase.getListNft(mockWalletId("wallet-a"), null) } returns flowOf(emptyList())
        every { getListNftCase.getListNft(mockWalletId("wallet-b"), null) } returns flowOf(emptyList())

        val subject = GetNftCollectionsImpl(sessionRepository, getListNftCase)

        subject(null).first()
        sessions.value = sessionB
        subject(null).first()

        verify { getListNftCase.getListNft(mockWalletId("wallet-a"), null) }
        verify { getListNftCase.getListNft(mockWalletId("wallet-b"), null) }
    }
}
