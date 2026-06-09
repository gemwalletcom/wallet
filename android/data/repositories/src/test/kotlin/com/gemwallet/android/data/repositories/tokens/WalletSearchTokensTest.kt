package com.gemwallet.android.data.repositories.tokens

import com.gemwallet.android.application.assets.coordinators.GemSearch
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.service.store.database.SearchPriorityDao
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBasic
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Perpetual
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.PerpetualSearchData
import com.wallet.core.primitives.SearchResponse
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertTrue
import org.junit.Test

class WalletSearchTokensTest {

    private val tokensRepository = mockk<TokensRepository>(relaxed = true)
    private val gemSearch = mockk<GemSearch>()
    private val perpetualRepository = mockk<PerpetualRepository>(relaxed = true)
    private val searchPriorityDao = mockk<SearchPriorityDao>(relaxed = true)

    private val subject = WalletSearchTokens(
        tokensRepository = tokensRepository,
        gemSearch = gemSearch,
        perpetualRepository = perpetualRepository,
        searchPriorityDao = searchPriorityDao,
    )

    @Test
    fun search_ingestsPerpetualsAndStoresPerpPriority() = runTest {
        val perpAsset = mockAsset()
        val perpetual = Perpetual(
            id = PerpetualId(provider = PerpetualProvider.Hypercore, symbol = "BTC"),
            name = "Bitcoin",
            provider = PerpetualProvider.Hypercore,
            assetId = perpAsset.id,
            identifier = "0",
            price = 1.0,
            pricePercentChange24h = 0.0,
            openInterest = 0.0,
            volume24h = 1.0,
            funding = 0.0,
            maxLeverage = 1u,
            isIsolatedOnly = false,
        )
        coEvery {
            gemSearch.search(query = "btc", chains = emptyList(), tags = emptyList())
        } returns SearchResponse(
            assets = listOf(mockAssetBasic()),
            perpetuals = listOf(PerpetualSearchData(perpetual = perpetual, asset = perpAsset)),
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
        coVerify { searchPriorityDao.put(match { priorities -> priorities.any { it.type == "perpetual" } }) }
    }
}
