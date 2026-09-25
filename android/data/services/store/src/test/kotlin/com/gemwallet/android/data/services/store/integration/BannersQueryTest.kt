package com.gemwallet.android.data.services.store.integration

import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAsset
import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbBanner
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.BannersQuery
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class BannersQueryTest {
    private lateinit var database: GemDatabase
    private val asset = mockAsset(id = mockAssetId(chain = Chain.Tron), name = "Tron", symbol = "TRX", decimals = 6)
    private val tokenId = AssetId(Chain.Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t")
    private val warning = DbBanner(
        id = "tron-multisignature",
        walletId = "wallet-1",
        assetId = asset.id.toIdentifier(),
        state = BannerState.AlwaysActive,
        event = BannerEvent.AccountBlockedMultiSignature,
    )

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        database.assetsDao().insert(asset.toRecord())
        database.bannersDao().addBanners(
            listOf(
                warning,
                warning.copy(id = "other-wallet", walletId = "wallet-2"),
                warning.copy(id = "other-chain", assetId = Chain.Ethereum.string),
                warning.copy(id = "stake", walletId = null, event = BannerEvent.Stake),
                warning.copy(id = "token", assetId = tokenId.toIdentifier(), event = BannerEvent.ActivateAsset),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun nativeAssetListsTheWalletAndSharedBannersWithTheStoredAsset() = runBlocking(Dispatchers.IO) {
        val banners = BannersQuery(database.bannersDao())("wallet-1", asset.id).first()

        assertEquals(setOf(BannerEvent.AccountBlockedMultiSignature, BannerEvent.Stake), banners.map { it.event }.toSet())
        assertEquals(listOf(asset, asset), banners.map { it.asset })
    }

    @Test
    fun tokenAlsoListsItsChainBanners() = runBlocking(Dispatchers.IO) {
        val banners = BannersQuery(database.bannersDao())("wallet-1", tokenId).first()

        assertEquals(setOf(BannerEvent.AccountBlockedMultiSignature, BannerEvent.Stake, BannerEvent.ActivateAsset), banners.map { it.event }.toSet())
    }

    @Test
    fun withoutWalletListsOnlySharedBanners() = runBlocking(Dispatchers.IO) {
        val banners = BannersQuery(database.bannersDao())(null, asset.id).first()

        assertEquals(listOf(BannerEvent.Stake), banners.map { it.event })
    }
}
