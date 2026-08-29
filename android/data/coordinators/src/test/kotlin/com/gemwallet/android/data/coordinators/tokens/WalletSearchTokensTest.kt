package com.gemwallet.android.data.coordinators.tokens

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.model.Session
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemSearchScope
import uniffi.gemstone.GemSearchService

class WalletSearchTokensTest {
    private val searchTokens = mockk<SearchTokensImpl>(relaxed = true)
    private val searchService = mockk<GemSearchService>()
    private val sessionRepository = mockk<SessionRepository>()
    private val subject = WalletSearchTokens(searchTokens, searchService, sessionRepository)

    @Test
    fun search_delegatesWalletAndScopeToCore() = runTest {
        val wallet = mockWallet()
        every { sessionRepository.session() } returns MutableStateFlow(Session(wallet = wallet, currency = Currency.USD))
        coEvery { searchService.search(wallet.toJson(), "btc", GemSearchScope.List("stocks"), Currency.USD.toJson()) } returns true

        val result = subject.search("btc", Currency.USD, emptyList(), WalletSearchTag.List("stocks"))

        assertTrue(result)
        coVerify(exactly = 1) { searchService.search(wallet.toJson(), "btc", GemSearchScope.List("stocks"), Currency.USD.toJson()) }
    }

    @Test
    fun search_returnsFalseWithoutSession() = runTest {
        every { sessionRepository.session() } returns MutableStateFlow(null)

        assertFalse(subject.search("btc", Currency.USD, emptyList()))
        coVerify(exactly = 0) { searchService.search(any(), any(), any(), any()) }
    }
}
