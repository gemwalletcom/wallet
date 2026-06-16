package com.gemwallet.android.data.repositories.tokens

import com.gemwallet.android.application.assets.coordinators.GemSearch
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBasic
import com.gemwallet.android.testkit.mockPerpetual
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualSearchData
import com.wallet.core.primitives.SearchResponse
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertTrue
import org.junit.Test

class WalletSearchTokensTest {

    private val tokensRepository = mockk<TokensRepository>(relaxed = true)
    private val gemSearch = mockk<GemSearch>()
    private val perpetualRepository = mockk<PerpetualRepository>(relaxed = true)
    private val searchDao = mockk<SearchDao>(relaxed = true)

    private val subject = WalletSearchTokens(
        tokensRepository = tokensRepository,
        gemSearch = gemSearch,
        perpetualRepository = perpetualRepository,
        searchDao = searchDao,
    )

    @Test
    fun search_ingestsPerpetualsAndStoresPerpPriority() = runTest {
        coEvery {
            gemSearch.search(query = "btc", chains = emptyList(), tags = emptyList())
        } returns SearchResponse(
            assets = listOf(mockAssetBasic()),
            perpetuals = listOf(PerpetualSearchData(perpetual = mockPerpetual(), asset = mockAsset())),
            nfts = emptyList(),
        )

        val result = subject.search(
            query = "btc",
            currency = Currency.USD,
            chains = emptyList(),
            tags = emptyList(),
        )

        assertTrue(result)
        coVerify { perpetualRepository.putPerpetuals(any()) }
        coVerify { searchDao.put(match { records -> records.any { it.perpetualId != null } }) }
    }

    @Test
    fun search_rethrowsCancellationWithoutStoring() = runTest {
        coEvery { gemSearch.search(any(), any(), any()) } throws CancellationException("cancelled")

        val result = runCatching {
            subject.search(query = "btc", currency = Currency.USD, chains = emptyList(), tags = emptyList())
        }

        assertTrue(result.exceptionOrNull() is CancellationException)
        coVerify(exactly = 0) { tokensRepository.updateAssets(any(), any()) }
        coVerify(exactly = 0) { searchDao.put(any()) }
        coVerify(exactly = 0) { searchDao.deleteAssets(any()) }
    }
}
