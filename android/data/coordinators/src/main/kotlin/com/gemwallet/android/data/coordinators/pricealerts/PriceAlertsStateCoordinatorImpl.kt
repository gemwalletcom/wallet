package com.gemwallet.android.data.coordinators.pricealerts

import com.gemwallet.android.application.pricealerts.coordinators.ExcludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.IncludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.PriceAlertsStateCoordinator
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.domains.pricealerts.values.PriceAlertsStateEvent
import com.gemwallet.android.domains.pricealerts.values.PriceAlertsStateEvent.Disable
import com.gemwallet.android.domains.pricealerts.values.PriceAlertsStateEvent.Enable
import com.gemwallet.android.domains.pricealerts.values.PriceAlertsStateEvent.Request
import com.wallet.core.primitives.AssetId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class PriceAlertsStateCoordinatorImpl(
    private val priceAlertRepository: PriceAlertRepository,
    private val includePriceAlert: IncludePriceAlert,
    private val excludePriceAlert: ExcludePriceAlert,
    private val syncDeviceInfo: SyncDeviceInfo,
) : PriceAlertsStateCoordinator {
    private val request = MutableStateFlow<Request?>(null)

    private val assetIdEnabled: Flow<Boolean> = request.flatMapLatest { request ->
        request?.assetId?.let {
            priceAlertRepository.getAssetPriceAlert(it).mapLatest { info -> info != null }
        } ?: flowOf(false)
    }

    override val priceAlertState: Flow<PriceAlertsStateEvent?> = combine(
        request,
        priceAlertRepository.isPriceAlertsEnabled(),
        assetIdEnabled,
    ) { request: Request?, priceAlertsEnabled: Boolean, assetState: Boolean ->
        if (request == null) {
            return@combine null
        }

        when {
            request.assetId != null && assetState -> Enable(request.assetId)
            request.assetId != null -> Disable(request.assetId)
            priceAlertsEnabled -> Enable()
            else -> Disable()
        }
    }

    private suspend fun enablePriceAlerts(assetId: AssetId?) {
        if (assetId == null) {
            if (priceAlertRepository.isPriceAlertsEnabled().firstOrNull() != true) {
                priceAlertRepository.togglePriceAlerts(true)
                syncDeviceInfo.syncDeviceInfo()
            }
        } else {
            includePriceAlert(assetId)
        }
    }

    private suspend fun disablePriceAlerts(assetId: AssetId?) {
        if (assetId == null) {
            priceAlertRepository.togglePriceAlerts(false)
            syncDeviceInfo.syncDeviceInfo()
        } else {
            excludePriceAlert(assetId)
        }
    }

    override suspend fun priceAlertState(state: PriceAlertsStateEvent) {
        when (state) {
            is Request -> request.value = state
            is Enable -> enablePriceAlerts(state.assetId)
            is Disable -> disablePriceAlerts(state.assetId)
        }
    }
}
