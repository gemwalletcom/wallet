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
import uniffi.gemstone.StakeConfig
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
    fun `stake counts pending and rewards on delegating chains`() {
        val cosmos = model(
            mockAsset(chain = Chain.Cosmos, symbol = "ATOM", decimals = 6),
            staked = "1000000",
            pending = "2000000",
            rewards = "3000000",
        )
        val solana = model(mockAsset(chain = Chain.Solana, symbol = "SOL", decimals = 6), staked = "4000000", rewards = "1000000")

        assertEquals("6 ATOM", cosmos.accountInfoUIModel.stake)
        assertEquals("5 SOL", solana.accountInfoUIModel.stake)
    }

    @Test
    fun `stake counts the frozen balance and ignores votes on tron`() {
        val tron = model(
            mockAsset(chain = Chain.Tron, symbol = "TRX", decimals = 6),
            frozen = "1000000",
            locked = "2000000",
            staked = "9000000",
            rewards = "3000000",
        )

        assertEquals("6 TRX", tron.accountInfoUIModel.stake)
    }

    @Test
    fun `stake is hidden unless staking is enabled or a position is held`() {
        val enabled = model(mockAsset(chain = Chain.Cosmos), metadata = mockAssetMetaData(isStakeEnabled = true, stakingApr = 5.0))
        val disabled = model(mockAsset(chain = Chain.Cosmos))
        val disabledWithRewards = model(mockAsset(chain = Chain.Cosmos, symbol = "ATOM", decimals = 6), rewards = "2000000")
        val nonStaking = model(mockAsset(chain = Chain.Bitcoin), metadata = mockAssetMetaData(isStakeEnabled = true))

        assertTrue(enabled.accountInfoUIModel.stake.startsWith("APR"))
        assertEquals("", disabled.accountInfoUIModel.stake)
        assertEquals("2 ATOM", disabledWithRewards.accountInfoUIModel.stake)
        assertFalse(disabledWithRewards.accountInfoUIModel.stake.startsWith("APR"))
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
        frozen: String = "0",
        locked: String = "0",
        staked: String = "0",
        pending: String = "0",
        rewards: String = "0",
        reserved: String = "0",
        metadata: AssetMetaData? = null,
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
        return AssetInfoUIModelFactory(swapService, StakeConfig()).create(
            ChainAssetInfo(assetInfo = assetInfo, feeAssetInfo = assetInfo),
            explorerName = "Explorer",
            walletType = WalletType.Multicoin,
        )
    }
}
