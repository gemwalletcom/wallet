package com.gemwallet.android.features.asset.viewmodels.details.models

import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetMetaData
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import io.mockk.every
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import io.mockk.mockk
import uniffi.gemstone.GemSwapPairSuggestion
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemSwapServiceInterface

class AssetInfoUIModelFactoryTest {

    @Before
    fun setUp() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { Chain.Cosmos.asset() } returns mockAsset(chain = Chain.Cosmos, name = "Cosmos")
        every { Chain.Solana.asset() } returns mockAsset(chain = Chain.Solana, name = "Solana")
        every { Chain.Tron.asset() } returns mockAsset(chain = Chain.Tron, name = "Tron")
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
    fun `stake renders the value the stake service reports`() {
        val cosmos = model(
            mockAsset(chain = Chain.Cosmos, symbol = "ATOM", decimals = 6),
            showsStake = true,
            stakedValue = "6000000",
        )

        assertEquals("6 ATOM", cosmos.accountInfoUIModel.stake)
    }

    @Test
    fun `stake is hidden when the stake service hides the row`() {
        val hidden = model(mockAsset(chain = Chain.Cosmos), showsStake = false, stakedValue = "6000000")

        assertEquals("", hidden.accountInfoUIModel.stake)
    }

    @Test
    fun `stake falls back to the apr when no position is held`() {
        val apr = model(
            mockAsset(chain = Chain.Cosmos),
            metadata = mockAssetMetaData(isStakeEnabled = true, stakingApr = 5.0),
            showsStake = true,
            stakedValue = "0",
        )
        val position = model(
            mockAsset(chain = Chain.Cosmos, symbol = "ATOM", decimals = 6),
            metadata = mockAssetMetaData(isStakeEnabled = true, stakingApr = 5.0),
            showsStake = true,
            stakedValue = "2000000",
        )

        assertTrue(apr.accountInfoUIModel.stake.startsWith("APR"))
        assertEquals("2 ATOM", position.accountInfoUIModel.stake)
        assertFalse(position.accountInfoUIModel.stake.startsWith("APR"))
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
        frozen: String = "0",
        locked: String = "0",
        staked: String = "0",
        pending: String = "0",
        rewards: String = "0",
        reserved: String = "0",
        metadata: AssetMetaData = mockAssetMetaData(),
        showsStake: Boolean = false,
        stakedValue: String = "0",
    ): AssetInfoUIModel {
        val balance = AssetBalance.create(
            asset,
            available = available,
            frozen = frozen,
            locked = locked,
            staked = staked,
            pending = pending,
            rewards = rewards,
            reserved = reserved,
        )
        val assetInfo = mockAssetInfo(asset = asset, owner = null, balance = balance, metadata = metadata)
        val swapService = mockk<GemSwapServiceInterface> { every { pairForAsset(any(), any()) } answers { GemSwapPairSuggestion(firstArg(), null) } }
        val stakeService = mockk<GemStakeServiceInterface> {
            every { showsStakeBalance(any(), any(), any()) } returns showsStake
            every { stakedValue(any(), any()) } returns stakedValue
        }
        return AssetInfoUIModelFactory(swapService, stakeService).create(
            ChainAssetInfo(assetInfo = assetInfo, feeAssetInfo = assetInfo),
            explorerName = "Explorer",
            walletType = WalletType.Multicoin,
            explorerAddressUrl = null,
            explorerTokenUrl = null,
        )
    }
}
