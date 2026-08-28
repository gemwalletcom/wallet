package com.gemwallet.android.features.settings.price_alerts.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.device.cases.EnableDevicePush
import com.gemwallet.android.application.pricealerts.cases.ExcludePriceAlert
import com.gemwallet.android.application.pricealerts.cases.GetAssetPriceAlertState
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlertsEnabled
import com.gemwallet.android.application.pricealerts.cases.IncludePriceAlert
import com.gemwallet.android.application.pricealerts.cases.SetPriceAlertsEnabled
import com.gemwallet.android.application.pricealerts.cases.UpdatePriceAlerts
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ui.models.navigation.RouteArgument
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
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject
import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PriceAlertViewModel @Inject constructor(
    getPriceAlerts: GetPriceAlerts,
    private val getPriceAlertsEnabled: GetPriceAlertsEnabled,
    private val getAssetPriceAlertState: GetAssetPriceAlertState,
    private val setPriceAlertsEnabled: SetPriceAlertsEnabled,
    private val updatePriceAlerts: UpdatePriceAlerts,
    private val assetsRepository: AssetsRepository,
    private val includePriceAlert: IncludePriceAlert,
    private val excludePriceAlert: ExcludePriceAlert,
    private val enableDevicePush: EnableDevicePush,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val refreshState = MutableStateFlow(false)

    val assetId = savedStateHandle.getStateFlow<String?>(RouteArgument.AssetId.key, null)
        .mapLatest { it?.toAssetId() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetInfo = assetId.flatMapLatest { id ->
        if (id != null) assetsRepository.getTokenInfo(id) else flowOf(null)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val data = assetId.flatMapLatest { getPriceAlerts(it) }
        .mapLatest { getPriceAlerts.groupByTargetAndAsset(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyMap())

    val priceAlertEnabled = assetId.flatMapLatest { id ->
        if (id == null) {
            getPriceAlertsEnabled.isPriceAlertsEnabled()
        } else {
            getAssetPriceAlertState.isAssetPriceAlertEnabled(id)
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isRefreshing = refreshState.asStateFlow()

    init {
        val initialAssetId = savedStateHandle.get<String?>(RouteArgument.AssetId.key)?.toAssetId()
        viewModelScope.launch(Dispatchers.IO) {
            runCatchingCancellable {
                if (initialAssetId != null) {
                    updatePriceAlerts.update(initialAssetId)
                } else {
                    updatePriceAlerts.update()
                }
            }.onFailure { Log.e(TAG, "price alerts sync failed", it) }
        }
    }

    fun refresh() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                refreshState.value = true
                val assetId = assetId.value
                runCatchingCancellable {
                    if (assetId != null) {
                        updatePriceAlerts.update(assetId)
                    } else {
                        updatePriceAlerts.update()
                    }
                }.onFailure { Log.e(TAG, "price alerts refresh failed", it) }
            } finally {
                refreshState.value = false
            }
        }
    }

    fun isAssetManage(): Boolean = assetId.value != null

    fun togglePriceAlerts(enable: Boolean) {
        viewModelScope.launch {
            setPriceAlertsEnabled(enable)
        }
    }

    fun onPushNotificationGranted() = viewModelScope.launch(Dispatchers.IO) {
        enableDevicePush()
    }

    fun toggleAutoAlert(enabled: Boolean) = viewModelScope.launch(Dispatchers.IO) {
        val assetId = assetId.value ?: return@launch
        if (enabled) {
            includePriceAlert(assetId)
        } else {
            val autoAlert = data.value[null]?.firstOrNull() ?: return@launch
            excludePriceAlert(autoAlert.priceAlert)
        }
    }

    fun excludeAsset(priceAlertId: String) = viewModelScope.launch(Dispatchers.IO) {
        val alert = data.value.values.flatten().firstOrNull { it.id == priceAlertId } ?: return@launch
        excludePriceAlert(alert.priceAlert)
    }

    fun includeAsset(assetId: AssetId, callback: (Asset) -> Unit) = viewModelScope.launch(Dispatchers.IO) {
        includePriceAlert(assetId)

        val assetInfo = assetsRepository.getTokenInfo(assetId).firstOrNull() ?: return@launch
        withContext(Dispatchers.Main) { callback(assetInfo.asset) }
    }

    private companion object {
        const val TAG = "PriceAlerts"
    }

}
