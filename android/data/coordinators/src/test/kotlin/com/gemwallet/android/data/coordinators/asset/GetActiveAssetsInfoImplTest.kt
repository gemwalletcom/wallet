package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregates
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class GetActiveAssetsInfoImplTest {
    private val assets = listOf(
        mockAssetInfo(asset = mockAsset(Chain.Bitcoin)).copy(price = mockAssetPriceInfo(price = 50000.0, priceChangePercentage24h = 2.5)),
        mockAssetInfo(asset = mockAsset(Chain.Ethereum)).copy(price = mockAssetPriceInfo(price = 3000.0, currency = Currency.EUR)),
        mockAssetInfo(asset = mockAsset(Chain.Solana)),
    )

    private val getWalletAssets = object : GetWalletAssets {
        override fun invoke(): Flow<List<AssetInfo>> = flowOf(assets)
        override fun invoke(walletId: WalletId): Flow<List<AssetInfo>> = flowOf(assets)
        override fun invoke(assetIds: List<AssetId>): Flow<List<AssetInfo>> = flowOf(assets)
        override fun byIdentifiers(assetIds: List<String>): Flow<List<AssetInfo>> = flowOf(assets)
    }

    private val subject = GetActiveAssetsInfoImpl(getWalletAssets)

    @Test
    fun emitsFormattedRowsForEveryWalletAsset() = runTest {
        val rows = subject.getAssetsInfo(hideBalance = false).first()

        assertEquals(assets.toAssetInfoDataAggregates(hideBalance = false), rows)
        assertEquals("\$50,000.00", rows.first().price?.valueFormatted)
        assertEquals("+2.50%", rows.first().price?.changePercentageFormatted)
    }

    @Test
    fun hidesBalancesWhenAsked() = runTest {
        val rows = subject.getAssetsInfo(hideBalance = true).first()

        assertEquals(assets.toAssetInfoDataAggregates(hideBalance = true), rows)
        assertEquals(listOf("*****", "*****", "*****"), rows.map { it.balance })
    }
}
