package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.ExcludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.IncludePriceAlert
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.domains.pricealerts.values.PriceAlertsStateEvent
import com.gemwallet.android.model.PriceAlertInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.yield
import org.junit.Assert.assertEquals
import org.junit.Test

class PriceAlertsStateCoordinatorImplTest {
    @Test
    fun enable_handlesIncludeOnceWhenStateFlowsReemit() = runBlocking {
        val assetId = AssetId(Chain.SmartChain)
        val priceAlertsEnabled = MutableStateFlow(false)
        val assetPriceAlert = MutableStateFlow<PriceAlertInfo?>(null)
        val includePriceAlert = RecordingIncludePriceAlert(assetPriceAlert)

        val coordinator = PriceAlertsStateCoordinatorImpl(
            priceAlertRepository = FakePriceAlertRepository(
                priceAlertsEnabled = priceAlertsEnabled,
                assetPriceAlert = assetPriceAlert,
            ),
            includePriceAlert = includePriceAlert,
            excludePriceAlert = RecordingExcludePriceAlert(assetPriceAlert),
            syncDeviceInfo = object : SyncDeviceInfo {
                override suspend fun syncDeviceInfo() = Unit
            },
        )

        val job = launch { coordinator.priceAlertState.collectLatest { } }
        yield()

        coordinator.priceAlertState(PriceAlertsStateEvent.Enable(assetId))
        delay(100)

        assertEquals(1, includePriceAlert.calls)

        job.cancel()
    }

    @Test
    fun disable_handlesExcludeOnceWhenStateFlowsReemit() = runBlocking {
        val assetId = AssetId(Chain.SmartChain)
        val priceAlertsEnabled = MutableStateFlow(true)
        val assetPriceAlert = MutableStateFlow<PriceAlertInfo?>(
            PriceAlertInfo(
                id = 1,
                priceAlert = PriceAlert(
                    assetId = assetId,
                    currency = Currency.USD.string,
                    price = null,
                    pricePercentChange = null,
                    priceDirection = null,
                )
            )
        )
        val excludePriceAlert = RecordingExcludePriceAlert(assetPriceAlert)

        val coordinator = PriceAlertsStateCoordinatorImpl(
            priceAlertRepository = FakePriceAlertRepository(
                priceAlertsEnabled = priceAlertsEnabled,
                assetPriceAlert = assetPriceAlert,
            ),
            includePriceAlert = RecordingIncludePriceAlert(assetPriceAlert),
            excludePriceAlert = excludePriceAlert,
            syncDeviceInfo = object : SyncDeviceInfo {
                override suspend fun syncDeviceInfo() = Unit
            },
        )

        val job = launch { coordinator.priceAlertState.collectLatest { } }
        yield()

        coordinator.priceAlertState(PriceAlertsStateEvent.Disable(assetId))
        delay(100)

        assertEquals(1, excludePriceAlert.calls)

        job.cancel()
    }

    private class FakePriceAlertRepository(
        private val priceAlertsEnabled: MutableStateFlow<Boolean>,
        private val assetPriceAlert: MutableStateFlow<PriceAlertInfo?>,
    ) : PriceAlertRepository {
        override fun isPriceAlertsEnabled(): Flow<Boolean> = priceAlertsEnabled

        override suspend fun togglePriceAlerts(enabled: Boolean) {
            priceAlertsEnabled.value = enabled
        }

        override suspend fun hasAssetPriceAlerts(assetId: AssetId): Boolean = assetPriceAlert.value != null

        override suspend fun getSamePriceAlert(priceAlert: PriceAlert): PriceAlertInfo? = null

        override fun getPriceAlerts(assetId: AssetId?): Flow<List<PriceAlertInfo>> = flowOf(emptyList())

        override fun getAssetPriceAlert(assetId: AssetId): Flow<PriceAlertInfo?> = assetPriceAlert

        override suspend fun addPriceAlert(priceAlert: PriceAlert) = Unit

        override suspend fun updatePriceAlerts(alerts: List<PriceAlert>) = Unit

        override suspend fun updateAssetPriceAlerts(assetId: AssetId, alerts: List<PriceAlert>) = Unit

        override suspend fun getPriceAlert(priceAlertId: Int): PriceAlertInfo? = null

        override suspend fun disable(priceAlertId: Int) = Unit

        override suspend fun enable(priceAlertId: Int) = Unit
    }

    private class RecordingIncludePriceAlert(
        private val assetPriceAlert: MutableStateFlow<PriceAlertInfo?>,
    ) : IncludePriceAlert {
        var calls = 0

        override suspend fun invoke(
            assetId: AssetId,
            currency: Currency?,
            price: Double?,
            percentage: Double?,
            direction: PriceAlertDirection?,
        ) {
            calls += 1
            assetPriceAlert.value = PriceAlertInfo(
                id = 1,
                priceAlert = PriceAlert(
                    assetId = assetId,
                    currency = currency?.string ?: Currency.USD.string,
                    price = price,
                    pricePercentChange = percentage,
                    priceDirection = direction,
                )
            )
        }
    }

    private class RecordingExcludePriceAlert(
        private val assetPriceAlert: MutableStateFlow<PriceAlertInfo?>,
    ) : ExcludePriceAlert {
        var calls = 0

        override suspend fun invoke(priceAlertId: Int) = Unit

        override suspend fun invoke(
            assetId: AssetId,
            currency: Currency?,
            price: Double?,
            percentage: Double?,
            direction: PriceAlertDirection?,
        ) {
            calls += 1
            assetPriceAlert.value = null
        }
    }
}
