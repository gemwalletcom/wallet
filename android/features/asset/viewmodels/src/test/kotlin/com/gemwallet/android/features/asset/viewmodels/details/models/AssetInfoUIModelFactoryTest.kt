package com.gemwallet.android.features.asset.viewmodels.details.models

import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

class AssetInfoUIModelFactoryTest {

    @Before
    fun setUp() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { Chain.Cosmos.asset() } returns mockAsset(chain = Chain.Cosmos, name = "Cosmos")
        every { Chain.Bitcoin.asset() } returns mockAsset(chain = Chain.Bitcoin, name = "Bitcoin")
    }

    @After
    fun tearDown() = unmockkStatic("com.gemwallet.android.ext.ChainKt")

    @Test
    fun `name uses chain asset for native and own name for token`() {
        val native = model(mockAsset(chain = Chain.Cosmos, name = "Renamed Cosmos"))
        val token = model(mockAsset(chain = Chain.Cosmos, name = "Token", type = AssetType.TOKEN))

        assertEquals("Cosmos", native.name)
        assertEquals("Token", token.name)
    }

    @Test
    fun `available is hidden when equal to total and shown otherwise`() {
        val whole = model(mockAsset(chain = Chain.Cosmos), available = "3000000")
        val partial = model(mockAsset(chain = Chain.Cosmos), available = "1000000", staked = "2000000")

        assertEquals("", whole.accountInfoUIModel.available)
        assertTrue(partial.accountInfoUIModel.available.isNotEmpty())
    }

    @Test
    fun `stake formats balance, falls back to apr, and is empty off staking chains`() {
        val withStake = model(mockAsset(chain = Chain.Cosmos), available = "1000000", staked = "2000000")
        val withoutStake = model(mockAsset(chain = Chain.Cosmos), available = "1000000")
        val nonStaking = model(mockAsset(chain = Chain.Bitcoin), available = "100000000")

        assertTrue(withStake.accountInfoUIModel.stake.isNotEmpty())
        assertFalse(withStake.accountInfoUIModel.stake.startsWith("APR"))
        assertTrue(withoutStake.accountInfoUIModel.stake.startsWith("APR"))
        assertEquals("", nonStaking.accountInfoUIModel.stake)
    }

    @Test
    fun `reserved is shown only when non zero`() {
        val reserved = model(mockAsset(chain = Chain.Bitcoin), available = "100000000", reserved = "500000")
        val noReserved = model(mockAsset(chain = Chain.Bitcoin), available = "100000000")

        assertTrue(reserved.accountInfoUIModel.reserved.isNotEmpty())
        assertEquals("", noReserved.accountInfoUIModel.reserved)
    }

    private fun model(
        asset: Asset,
        available: String = "0",
        staked: String = "0",
        reserved: String = "0",
    ): AssetInfoUIModel {
        val balance = AssetBalance.create(asset, available = available, staked = staked, reserved = reserved)
        val assetInfo = mockAssetInfo(asset = asset, owner = null, balance = balance)
        return AssetInfoUIModelFactory.create(
            ChainAssetInfo(assetInfo = assetInfo, feeAssetInfo = assetInfo),
            explorerName = "Explorer",
        )
    }
}
