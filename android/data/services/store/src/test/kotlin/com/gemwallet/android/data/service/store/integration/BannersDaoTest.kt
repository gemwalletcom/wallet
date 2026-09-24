package com.gemwallet.android.data.service.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.entities.DbBanner
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAssetTron
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.BannerState
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class BannersDaoTest {
    private lateinit var database: GemDatabase
    private val asset = mockAssetTron()
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
        database.bannersDao().saveBanner(warning)
    }

    @After
    fun tearDown() {
        database.close()
    }

    @Test
    fun walletWarningIncludesStoredAsset() = runBlocking(Dispatchers.IO) {
        val banner = database.bannersDao()
            .observeWalletBanners("wallet-1", listOf(BannerEvent.AccountBlockedMultiSignature))
            .first().single().toDTO()

        assertEquals(asset, banner.asset)
    }

    @Test
    fun nativeAssetWarningIncludesStoredAsset() = runBlocking(Dispatchers.IO) {
        val banner = database.bannersDao()
            .observeAssetBanners("wallet-1", asset.id.toIdentifier(), asset.id.toIdentifier())
            .first().single().toDTO()

        assertEquals(asset, banner.asset)
    }

    @Test
    fun tokenLoadsCandidatesForTheSameWalletAndChain() = runBlocking(Dispatchers.IO) {
        database.bannersDao().addBanners(
            listOf(
                warning.copy(id = "other-wallet", walletId = "wallet-2"),
                warning.copy(id = "other-chain", assetId = Chain.Ethereum.string),
                warning.copy(id = "stake", walletId = null, event = BannerEvent.Stake),
            ),
        )
        val tokenId = AssetId(Chain.Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t")
        val banners = database.bannersDao()
            .observeAssetBanners("wallet-1", tokenId.toIdentifier(), asset.id.toIdentifier())
            .first().map { it.toDTO() }

        assertEquals(setOf(BannerEvent.AccountBlockedMultiSignature, BannerEvent.Stake), banners.map { it.event }.toSet())
        assertEquals(listOf(asset, asset), banners.map { it.asset })
    }

    @Test
    fun walletKeepsAssetlessOnboarding() = runBlocking(Dispatchers.IO) {
        database.bannersDao().saveBanner(warning.copy(id = "onboarding", assetId = null, event = BannerEvent.Onboarding))
        val banner = database.bannersDao()
            .observeWalletBanners("wallet-1", listOf(BannerEvent.Onboarding))
            .first().single().toDTO()

        assertEquals(BannerEvent.Onboarding, banner.event)
        assertNull(banner.asset)
    }
}
