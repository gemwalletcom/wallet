package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregates
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetPrice
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotSame
import org.junit.Assert.assertSame
import org.junit.Test
import uniffi.gemstone.GemLocalizedText

class GetActiveAssetsInfoImplTest {
    private val assets = listOf(
        mockAssetInfo(asset = mockAsset(id = mockAssetId(chain = Chain.Bitcoin)), price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 50000.0, priceChangePercentage24h = 2.5))),
        mockAssetInfo(asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum)), price = mockAssetPriceInfo(currency = Currency.EUR, price = mockAssetPrice(price = 3000.0))),
        mockAssetInfo(asset = mockAsset(id = mockAssetId(chain = Chain.Solana))),
    )

    private val walletAssets = MutableStateFlow(assets)

    private val getWalletAssets = object : GetWalletAssets {
        override fun invoke(): StateFlow<List<AssetInfo>> = walletAssets
    }

    private val hideBalances = MutableStateFlow(false)

    private fun subject(hideBalance: Boolean, scope: CoroutineScope) = GetActiveAssetsInfoImpl(
        getWalletAssets = getWalletAssets,
        userConfig = mockk<UserConfig> { every { isHideBalances() } returns flowOf(hideBalance) },
        scope = scope,
    )

    private fun observed(scope: CoroutineScope) = GetActiveAssetsInfoImpl(
        getWalletAssets = getWalletAssets,
        userConfig = mockk<UserConfig> { every { isHideBalances() } returns hideBalances },
        scope = scope,
    )

    @Test
    fun emitsFormattedRowsForEveryWalletAsset() = runTest {
        val rows = subject(hideBalance = false, scope = backgroundScope).assetsInfo().first { it.isNotEmpty() }

        assertEquals(assets.toAssetInfoDataAggregates(hideBalance = false), rows)
        assertEquals("\$50,000.00", rows.first().priceText)
        assertEquals("+2.50%", rows.first().changeText)
    }

    @Test
    fun onlyTheChangedRowIsRebuilt() = runTest {
        val subject = observed(backgroundScope)
        val first = subject.assetsInfo().first { it.isNotEmpty() }

        walletAssets.value = assets.mapIndexed { index, item ->
            if (index ==
                0
            ) {
                item.copy(price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 51000.0, priceChangePercentage24h = 2.5)))
            } else {
                item.copy(balance = item.balance.copy(balance = item.balance.balance.copy()))
            }
        }
        val second = subject.assetsInfo().first { it.first().priceText == "\$51,000.00" }

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
        val hidden = subject.assetsInfo().first { it.first().hideBalance }

        assertNotSame(visible[0], hidden[0])
        assertEquals(listOf(true, true, true), hidden.map { it.hideBalance })
    }

    @Test
    fun hidesBalancesWhenAsked() = runTest {
        val rows = subject(hideBalance = true, scope = backgroundScope).assetsInfo().first { it.isNotEmpty() }

        assertEquals(assets.toAssetInfoDataAggregates(hideBalance = true), rows)
        assertEquals(listOf(true, true, true), rows.map { it.hideBalance })
    }
}

private val AssetInfoDataAggregate.priceText: String?
    get() = (row.subtitle?.text as? GemLocalizedText.Number)?.number?.text()

private val AssetInfoDataAggregate.changeText: String?
    get() = (row.subtitleExtra?.text as? GemLocalizedText.Number)?.number?.text()
