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
    fun `balance rows render the core rows with the apr standing in for an empty stake`() {
        val position = model(
            mockAsset(chain = Chain.Cosmos, symbol = "ATOM", decimals = 6),
            metadata = mockAssetMetaData(isStakeEnabled = true, stakingApr = 5.0),
            available = "1000000",
            staked = "2000000",
            reserved = "500000",
        ).accountInfoUIModel.balances
        assertEquals(
            listOf(AssetInfoUIModel.BalanceViewType.Available, AssetInfoUIModel.BalanceViewType.Stake, AssetInfoUIModel.BalanceViewType.Reserved),
            position.map { it.type },
        )
        assertEquals(listOf("1 ATOM", "2 ATOM", "0.5 ATOM"), position.map { it.value })

        val apr = model(mockAsset(chain = Chain.Cosmos), metadata = mockAssetMetaData(isStakeEnabled = true, stakingApr = 5.0)).accountInfoUIModel.balances
        assertEquals(listOf(AssetInfoUIModel.BalanceViewType.Stake), apr.map { it.type })
        assertTrue(apr.single().value.startsWith("APR"))

        assertTrue(model(mockAsset(chain = Chain.Bitcoin), available = "100000000").accountInfoUIModel.balances.isEmpty())
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
        return AssetInfoUIModelFactory(swapService).create(
            ChainAssetInfo(assetInfo = assetInfo, feeAssetInfo = assetInfo),
            explorerName = "Explorer",
            walletType = WalletType.Multicoin,
            explorerAddressUrl = null,
            explorerTokenUrl = null,
        )
    }
}
