package com.gemwallet.android.features.settings.price_alerts.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.device.cases.EnableDevicePush
import com.gemwallet.android.application.pricealerts.cases.GetAssetPriceAlertState
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
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
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject
import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import uniffi.gemstone.GemPriceAlertService

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PriceAlertViewModel @Inject constructor(
    getPriceAlerts: GetPriceAlerts,
    private val getAssetPriceAlertState: GetAssetPriceAlertState,
    private val getAssetTokenInfo: GetAssetTokenInfo,
    private val enableDevicePush: EnableDevicePush,
    private val service: GemPriceAlertService,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val refreshState = MutableStateFlow(false)
    private val alertsEnabled = MutableStateFlow(service.isEnabled())

    val assetId = savedStateHandle.getStateFlow<String?>(RouteArgument.AssetId.key, null)
        .mapLatest { it?.toAssetId() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetInfo = assetId.flatMapLatest { id ->
        if (id != null) getAssetTokenInfo(id) else flowOf(null)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val data = assetId.flatMapLatest { getPriceAlerts(it) }
        .mapLatest { getPriceAlerts.groupByTargetAndAsset(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyMap())

    val priceAlertEnabled = assetId.flatMapLatest { id ->
        if (id == null) {
            alertsEnabled
        } else {
            getAssetPriceAlertState.isAssetPriceAlertEnabled(id)
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isRefreshing = refreshState.asStateFlow()

    init {
        val initialAssetId = savedStateHandle.get<String?>(RouteArgument.AssetId.key)?.toAssetId()
        viewModelScope.launch(Dispatchers.IO) {
            runCatchingCancellable { service.sync(initialAssetId?.toIdentifier()) }
                .onFailure { Log.e(TAG, "price alerts sync failed", it) }
        }
    }

    fun refresh() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                refreshState.value = true
                runCatchingCancellable { service.sync(assetId.value?.toIdentifier()) }
                    .onFailure { Log.e(TAG, "price alerts refresh failed", it) }
            } finally {
                refreshState.value = false
            }
        }
    }

    fun isAssetManage(): Boolean = assetId.value != null

    fun togglePriceAlerts(enable: Boolean) = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { service.setEnabled(enable) }
            .onFailure { Log.e(TAG, "setting price alerts enabled failed", it) }
        alertsEnabled.update { service.isEnabled() }
    }

    fun onPushNotificationGranted() = viewModelScope.launch(Dispatchers.IO) {
        enableDevicePush()
    }

    fun toggleAutoAlert(enabled: Boolean) = viewModelScope.launch(Dispatchers.IO) {
        val assetId = assetId.value ?: return@launch
        setAutoAlert(assetId, enabled)
    }

    fun excludeAsset(priceAlertId: String) = viewModelScope.launch(Dispatchers.IO) {
        val alert = data.value.values.flatten().firstOrNull { it.id == priceAlertId } ?: return@launch
        runCatchingCancellable { service.deletePriceAlerts(listOf(alert.priceAlert.toJson())) }
            .onFailure { Log.e(TAG, "deleting the price alert for ${alert.assetId.toIdentifier()} failed", it) }
    }

    fun includeAsset(assetId: AssetId, callback: (Asset) -> Unit) = viewModelScope.launch(Dispatchers.IO) {
        setAutoAlert(assetId, true)

        val assetInfo = getAssetTokenInfo(assetId).firstOrNull() ?: return@launch
        withContext(Dispatchers.Main) { callback(assetInfo.asset) }
    }

    private suspend fun setAutoAlert(assetId: AssetId, enabled: Boolean) {
        runCatchingCancellable { service.setAutoAlert(assetId.toIdentifier(), enabled) }
            .onFailure { Log.e(TAG, "setting the auto price alert for ${assetId.toIdentifier()} failed", it) }
    }

    private companion object {
        const val TAG = "PriceAlerts"
    }

}
