package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbPerpetual
import com.gemwallet.android.data.service.store.database.entities.DbPerpetualData
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class GemstonePerpetualStoreTest {

    private val perpetualDao = mockk<PerpetualDao>()

    private val store = GemstonePerpetualStore(
        perpetualDao = perpetualDao,
        perpetualPositionDao = mockk<PerpetualPositionDao>(),
        balancesDao = mockk<BalancesDao>(),
        transactionRunner = mockk<StoreTransactionRunner>(),
    )

    @Test
    fun `an empty query observes every perpetual and a typed one observes the search rows`() = runTest {
        every { perpetualDao.getPerpetualsData() } returns flowOf(listOf(bitcoin(), ethereum()))
        every { perpetualDao.search("btc") } returns flowOf(listOf(bitcoin()))

        assertEquals(listOf("BTC-PERP", "ETH-PERP"), store.observePerpetuals(null).first().map { it.perpetual.identifier })
        assertEquals(listOf("BTC-PERP", "ETH-PERP"), store.observePerpetuals("  ").first().map { it.perpetual.identifier })
        assertEquals(listOf("BTC-PERP"), store.observePerpetuals(" btc ").first().map { it.perpetual.identifier })
    }

    private fun bitcoin() = perpetualData(identifier = "BTC-PERP", name = "Bitcoin Perpetual", asset = mockAsset())

    private fun ethereum() = perpetualData(identifier = "ETH-PERP", name = "Ethereum Perpetual", asset = mockAssetEthereum())

    private fun perpetualData(identifier: String, name: String, asset: Asset) = DbPerpetualData(
        perpetual = DbPerpetual(
            id = PerpetualId(PerpetualProvider.Hypercore, identifier),
            name = name,
            provider = PerpetualProvider.Hypercore,
            assetId = asset.id,
            identifier = identifier,
            price = 1.0,
            pricePercentChange24h = 0.0,
            openInterest = 0.0,
            volume24h = 0.0,
            funding = 0.0,
            maxLeverage = 1,
        ),
        asset = DbAsset(
            id = asset.id.toIdentifier(),
            name = asset.name,
            symbol = asset.symbol,
            decimals = asset.decimals,
            type = asset.type,
            chain = asset.id.chain,
        ),
    )
}
