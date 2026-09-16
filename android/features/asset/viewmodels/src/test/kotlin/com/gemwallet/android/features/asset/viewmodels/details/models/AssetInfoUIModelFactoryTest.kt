package com.gemwallet.android.features.asset.viewmodels.details.models

import com.gemwallet.android.ext.asset
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetMetaData
import com.gemwallet.android.testkit.mockChainAssetInfo
import com.gemwallet.android.testkit.mockGemAssetDetails
import com.gemwallet.android.testkit.mockGemAssetDetailsState
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.math.BigInteger

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
    fun `the row name is the title core decided`() {
        assertEquals("Renamed Cosmos", model(mockAssetInfo(asset = mockAsset(chain = Chain.Cosmos, name = "Renamed Cosmos"), owner = null)).name)
    }

    @Test
    fun `balance rows render the core rows with the apr standing in for an empty stake`() {
        val atom = mockAsset(chain = Chain.Cosmos, symbol = "ATOM", decimals = 6)
        val position = model(
            mockAssetInfo(
                asset = atom,
                owner = null,
                balance = AssetBalance.create(atom, available = BigInteger("1000000"), staked = BigInteger("2000000"), reserved = BigInteger("500000")),
                metadata = mockAssetMetaData(isStakeEnabled = true, stakingApr = 5.0),
            ),
        ).accountInfoUIModel.balances
        assertEquals(
            listOf(AssetInfoUIModel.BalanceViewType.Available, AssetInfoUIModel.BalanceViewType.Stake, AssetInfoUIModel.BalanceViewType.Reserved),
            position.map { it.type },
        )
        assertEquals(listOf("1 ATOM", "2 ATOM", "0.5 ATOM"), position.map { it.value })

        val apr = model(mockAssetInfo(asset = mockAsset(chain = Chain.Cosmos), owner = null, metadata = mockAssetMetaData(isStakeEnabled = true, stakingApr = 5.0))).accountInfoUIModel.balances
        assertEquals(listOf(AssetInfoUIModel.BalanceViewType.Stake), apr.map { it.type })
        assertTrue(apr.single().value.startsWith("APR"))

        val bitcoin = mockAsset()
        assertTrue(model(mockAssetInfo(asset = bitcoin, owner = null, balance = AssetBalance.create(bitcoin, available = BigInteger("100000000")))).accountInfoUIModel.balances.isEmpty())
    }

    private fun model(assetInfo: AssetInfo) = AssetInfoUIModelFactory().create(
        mockChainAssetInfo(assetInfo),
        mockGemAssetDetails(assetInfo.asset, mockGemAssetDetailsState(showsBanners = true)),
        banners = emptyList(),
    )
}
