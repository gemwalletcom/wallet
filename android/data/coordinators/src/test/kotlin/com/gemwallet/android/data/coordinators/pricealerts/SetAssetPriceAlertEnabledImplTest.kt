package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.ExcludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.IncludePriceAlert
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test

class SetAssetPriceAlertEnabledImplTest {

    @Test
    fun enable_includesAssetOnly() = runBlocking {
        val assetId = AssetId(Chain.SmartChain)
        val includePriceAlert = RecordingIncludePriceAlert()
        val excludePriceAlert = RecordingExcludePriceAlert()

        SetAssetPriceAlertEnabledImpl(includePriceAlert, excludePriceAlert).invoke(assetId, true)

        assertEquals(listOf(assetId), includePriceAlert.assetIds)
        assertEquals(emptyList<AssetId>(), excludePriceAlert.assetIds)
    }

    @Test
    fun disable_excludesAssetOnly() = runBlocking {
        val assetId = AssetId(Chain.SmartChain)
        val includePriceAlert = RecordingIncludePriceAlert()
        val excludePriceAlert = RecordingExcludePriceAlert()

        SetAssetPriceAlertEnabledImpl(includePriceAlert, excludePriceAlert).invoke(assetId, false)

        assertEquals(emptyList<AssetId>(), includePriceAlert.assetIds)
        assertEquals(listOf(assetId), excludePriceAlert.assetIds)
        assertEquals(0, excludePriceAlert.priceAlertCalls)
    }

    private class RecordingIncludePriceAlert : IncludePriceAlert {
        val assetIds = mutableListOf<AssetId>()

        override suspend fun invoke(
            assetId: AssetId,
            currency: Currency?,
            price: Double?,
            percentage: Double?,
            direction: PriceAlertDirection?,
        ) {
            assetIds.add(assetId)
        }
    }

    private class RecordingExcludePriceAlert : ExcludePriceAlert {
        val assetIds = mutableListOf<AssetId>()
        var priceAlertCalls = 0

        override suspend fun invoke(priceAlert: PriceAlert) {
            priceAlertCalls += 1
        }

        override suspend fun invoke(
            assetId: AssetId,
            currency: Currency?,
            price: Double?,
            percentage: Double?,
            direction: PriceAlertDirection?,
        ) {
            assetIds.add(assetId)
        }
    }
}
