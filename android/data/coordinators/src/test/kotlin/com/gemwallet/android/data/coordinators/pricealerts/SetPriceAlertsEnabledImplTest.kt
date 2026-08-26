package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.model.PriceAlertInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlert
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test

class SetPriceAlertsEnabledImplTest {

    @Test
    fun enable_whenAlreadyEnabled_skipsToggleAndDeviceSync() = runBlocking {
        val repository = FakePriceAlertRepository(enabled = true)
        val syncDevice = RecordingSyncDevice()

        SetPriceAlertsEnabledImpl(repository, syncDevice).invoke(true)

        assertEquals(0, repository.toggleCalls)
        assertEquals(0, syncDevice.calls)
        assertEquals(true, repository.enabled.value)
    }

    @Test
    fun enable_whenDisabled_togglesAndSyncsDevice() = runBlocking {
        val repository = FakePriceAlertRepository(enabled = false)
        val syncDevice = RecordingSyncDevice()

        SetPriceAlertsEnabledImpl(repository, syncDevice).invoke(true)

        assertEquals(1, repository.toggleCalls)
        assertEquals(1, syncDevice.calls)
        assertEquals(true, repository.enabled.value)
    }

    @Test
    fun disable_togglesAndSyncsDeviceEvenWhenAlreadyDisabled() = runBlocking {
        val enabledRepository = FakePriceAlertRepository(enabled = true)
        val enabledSync = RecordingSyncDevice()
        SetPriceAlertsEnabledImpl(enabledRepository, enabledSync).invoke(false)

        val disabledRepository = FakePriceAlertRepository(enabled = false)
        val disabledSync = RecordingSyncDevice()
        SetPriceAlertsEnabledImpl(disabledRepository, disabledSync).invoke(false)

        assertEquals(1, enabledRepository.toggleCalls)
        assertEquals(1, enabledSync.calls)
        assertEquals(false, enabledRepository.enabled.value)
        assertEquals(1, disabledRepository.toggleCalls)
        assertEquals(1, disabledSync.calls)
        assertEquals(false, disabledRepository.enabled.value)
    }

    private class RecordingSyncDevice : SyncDevice {
        var calls = 0

        override suspend fun syncDevice() {
            calls += 1
        }
    }

    private class FakePriceAlertRepository(enabled: Boolean) : PriceAlertRepository {
        val enabled = MutableStateFlow(enabled)
        var toggleCalls = 0

        override fun isPriceAlertsEnabled(): Flow<Boolean> = enabled

        override suspend fun togglePriceAlerts(enabled: Boolean) {
            toggleCalls += 1
            this.enabled.value = enabled
        }

        override suspend fun hasAssetPriceAlerts(assetId: AssetId): Boolean = false

        override suspend fun getSamePriceAlert(priceAlert: PriceAlert): PriceAlertInfo? = null

        override fun getPriceAlerts(assetId: AssetId?): Flow<List<PriceAlertInfo>> = flowOf(emptyList())

        override fun getPriceAlertAssetIds(): Flow<List<AssetId>> = flowOf(emptyList())

        override fun getAssetPriceAlert(assetId: AssetId): Flow<PriceAlertInfo?> = flowOf(null)

        override suspend fun addPriceAlert(priceAlert: PriceAlert) = Unit



        override suspend fun getPriceAlert(priceAlertId: Int): PriceAlertInfo? = null

        override suspend fun disable(priceAlertId: Int) = Unit

        override suspend fun enable(priceAlertId: Int) = Unit
    }
}
