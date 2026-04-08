package com.gemwallet.android.features.settings.price_alerts.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.pricealerts.coordinators.ExcludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.IncludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.PriceAlertsStateCoordinator
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.domains.pricealerts.values.PriceAlertsStateEvent
import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.withContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PriceAlertViewModel @Inject constructor(
    getPriceAlerts: GetPriceAlerts,
    private val priceAlertsStateCoordinator: PriceAlertsStateCoordinator,
    private val updatePriceAlerts: UpdatePriceAlerts,
    private val assetsRepository: AssetsRepository,
    private val includePriceAlert: IncludePriceAlert,
    private val excludePriceAlert: ExcludePriceAlert,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val refreshState = MutableStateFlow(false)

    val assetId = savedStateHandle.getStateFlow<String?>("assetId", null)
        .mapLatest { it?.toAssetId() }
        .onEach { priceAlertsStateCoordinator.priceAlertState(PriceAlertsStateEvent.Request(it)) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetInfo = assetId.flatMapLatest { id ->
        if (id != null) assetsRepository.getTokenInfo(id) else flowOf(null)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val data = assetId.flatMapLatest { getPriceAlerts(it) }
        .mapLatest { getPriceAlerts.groupByTargetAndAsset(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyMap())

    val priceAlertState = priceAlertsStateCoordinator.priceAlertState
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isRefreshing = refreshState.asStateFlow()

    init {
        val initialAssetId = savedStateHandle.get<String?>("assetId")?.toAssetId()
        viewModelScope.launch(Dispatchers.IO) {
            runCatching {
                if (initialAssetId != null) {
                    updatePriceAlerts.update(initialAssetId)
                } else {
                    updatePriceAlerts.update()
                }
            }
        }
    }

    fun refresh() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                refreshState.value = true
                val assetId = assetId.value
                runCatching {
                    if (assetId != null) {
                        updatePriceAlerts.update(assetId)
                    } else {
                        updatePriceAlerts.update()
                    }
                }
            } finally {
                refreshState.value = false
            }
        }
    }

    fun isAssetManage(): Boolean = assetId.value != null

    fun togglePriceAlerts(enable: Boolean) {
        viewModelScope.launch {
            val newState = if (enable) PriceAlertsStateEvent.Enable() else PriceAlertsStateEvent.Disable()
            priceAlertsStateCoordinator.priceAlertState(newState)
        }
    }

    fun toggleAutoAlert(enabled: Boolean) = viewModelScope.launch(Dispatchers.IO) {
        val assetId = assetId.value ?: return@launch
        if (enabled) {
            includePriceAlert(assetId)
        } else {
            val autoAlert = data.value[null]?.firstOrNull()
            autoAlert?.let { excludePriceAlert(it.id) }
        }
    }

    fun excludeAsset(priceAlertId: Int) = viewModelScope.launch(Dispatchers.IO) {
        excludePriceAlert(priceAlertId)
    }

    fun includeAsset(assetId: AssetId, callback: (Asset) -> Unit) = viewModelScope.launch(Dispatchers.IO) {
        includePriceAlert(assetId)

        val assetInfo = assetsRepository.getTokenInfo(assetId).firstOrNull() ?: return@launch
        withContext(Dispatchers.Main) { callback(assetInfo.asset) }
    }
}
