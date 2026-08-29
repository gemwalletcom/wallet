package com.gemwallet.android.data.coordinators.tokens

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBasic
import com.gemwallet.android.testkit.mockSession
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
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemSearchService

class SearchTokensImplTest {

    private val wallet = mockWallet(id = "wallet-1")
    private val sessionRepository = mockk<SessionRepository> {
        every { session() } returns MutableStateFlow(mockSession(wallet = wallet))
    }
    private val searchService = mockk<GemSearchService>()
    private val assetsService = mockk<GemAssetsService>(relaxed = true)

    private val subject = SearchTokensImpl(
        sessionRepository = sessionRepository,
        searchService = searchService,
        assetsService = assetsService,
    )

    @Test
    fun search_usesCoreSearchAssetsForTheSessionWallet() = runTest {
        coEvery { searchService.searchAssets(wallet.toJson(), "btc", Currency.USD.toJson()) } returns listOf(mockAssetBasic().toJson())
        coEvery { searchService.searchAssets(wallet.toJson(), "none", Currency.USD.toJson()) } returns emptyList()

        assertTrue(subject.search(query = "btc", currency = Currency.USD))
        assertFalse(subject.search(query = "none", currency = Currency.USD))
        assertFalse(subject.search(query = "", currency = Currency.USD))
    }

    @Test
    fun searchByAssetIds_syncsThemThroughCore() = runTest {
        val asset = mockAsset()

        val result = subject.search(assetIds = listOf(asset.id), currency = Currency.USD)

        assertTrue(result)
        coVerify(exactly = 1) { assetsService.syncAssets(listOf(asset.id.toIdentifier()), Currency.USD.toJson()) }
    }
}
