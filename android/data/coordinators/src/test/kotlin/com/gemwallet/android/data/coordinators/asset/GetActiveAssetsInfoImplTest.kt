package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregates
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockGemAssetRowStyle
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WalletId
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotSame
import org.junit.Assert.assertSame
import org.junit.Test
import uniffi.gemstone.GemAssetTitleStyle

class GetActiveAssetsInfoImplTest {
    private val assets = listOf(
        mockAssetInfo(asset = mockAsset(Chain.Bitcoin), price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 2.5)),
        mockAssetInfo(asset = mockAsset(Chain.Ethereum), price = mockAssetPriceInfo(price = 3000.0, currency = Currency.EUR)),
        mockAssetInfo(asset = mockAsset(Chain.Solana)),
    )

    private val walletAssets = MutableStateFlow(assets)

    private val getWalletAssets = object : GetWalletAssets {
        override fun invoke(): StateFlow<List<AssetInfo>> = walletAssets
        override fun invoke(walletId: WalletId): Flow<List<AssetInfo>> = walletAssets
        override fun invoke(assetIds: List<AssetId>): Flow<List<AssetInfo>> = walletAssets
        override fun byIdentifiers(assetIds: List<String>): Flow<List<AssetInfo>> = walletAssets
    }

    private val rowStyle = mockGemAssetRowStyle(title = GemAssetTitleStyle.CANONICAL_ASSET)

    private val hideBalances = MutableStateFlow(false)

    private fun subject(hideBalance: Boolean, scope: CoroutineScope) = GetActiveAssetsInfoImpl(
        getWalletAssets = getWalletAssets,
        userConfig = mockk<UserConfig> { every { isHideBalances() } returns flowOf(hideBalance) },
        rowStyle = rowStyle,
        scope = scope,
    )

    private fun observed(scope: CoroutineScope) = GetActiveAssetsInfoImpl(
        getWalletAssets = getWalletAssets,
        userConfig = mockk<UserConfig> { every { isHideBalances() } returns hideBalances },
        rowStyle = rowStyle,
        scope = scope,
    )

    @Test
    fun emitsFormattedRowsForEveryWalletAsset() = runTest {
        val rows = subject(hideBalance = false, scope = backgroundScope).assetsInfo().first { it.isNotEmpty() }

        assertEquals(assets.toAssetInfoDataAggregates(style = rowStyle, hideBalance = false), rows)
        assertEquals("\$50,000.00", rows.first().price.price?.text())
        assertEquals("+2.50%", rows.first().price.change?.text())
    }

    @Test
    fun onlyTheChangedRowIsRebuilt() = runTest {
        val subject = observed(backgroundScope)
        val first = subject.assetsInfo().first { it.isNotEmpty() }

        walletAssets.value = assets.mapIndexed { index, item ->
            if (index == 0) item.copy(price = mockAssetPriceInfo(price = 51000.0, priceChangePercentage24h = 2.5)) else item.copy(balance = item.balance.copy(balance = item.balance.balance.copy()))
        }
        val second = subject.assetsInfo().first { it.first().price.price?.text() == "\$51,000.00" }

        assertNotSame(first[0], second[0])
        assertSame(first[1], second[1])
        assertSame(first[2], second[2])
    }

    @Test
    fun aRemovedRowLeavesTheCache() = runTest {
        val subject = observed(backgroundScope)
        val first = subject.assetsInfo().first { it.isNotEmpty() }

        walletAssets.value = assets.drop(1)
        subject.assetsInfo().first { it.size == 2 }
        walletAssets.value = assets
        val restored = subject.assetsInfo().first { it.size == 3 }

        assertNotSame(first[0], restored[0])
        assertSame(first[1], restored[1])
    }

    @Test
    fun hidingBalancesCannotReuseTheVisibleRows() = runTest {
        val subject = observed(backgroundScope)
        val visible = subject.assetsInfo().first { it.isNotEmpty() }

        hideBalances.value = true
        val hidden = subject.assetsInfo().first { it.first().balance == "*****" }

        assertNotSame(visible[0], hidden[0])
        assertEquals(listOf("*****", "*****", "*****"), hidden.map { it.balance })
    }

    @Test
    fun hidesBalancesWhenAsked() = runTest {
        val rows = subject(hideBalance = true, scope = backgroundScope).assetsInfo().first { it.isNotEmpty() }

        assertEquals(assets.toAssetInfoDataAggregates(style = rowStyle, hideBalance = true), rows)
        assertEquals(listOf("*****", "*****", "*****"), rows.map { it.balance })
    }
}
