package com.gemwallet.android.features.settings.price_alerts.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.application.pricealerts.cases.GetAssetPriceAlertState
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetRowTitle
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemPriceAlertServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PriceAlertViewModel @Inject constructor(
    getPriceAlerts: GetPriceAlerts,
    private val getAssetPriceAlertState: GetAssetPriceAlertState,
    private val getAssetTokenInfo: GetAssetTokenInfo,
    private val service: GemPriceAlertServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val refreshState = MutableStateFlow(false)
    private val alertsEnabled = MutableStateFlow(service.isEnabled())

    val assetId = savedStateHandle.getStateFlow<String?>(RouteArgument.AssetId.key, null)
        .mapLatest { it?.toAssetId() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val asset = assetId.flatMapLatest { id ->
        if (id != null) getAssetTokenInfo(id) else flowOf(null)
    }
        .mapLatest { it?.toAssetInfoDataAggregate(GemAssetRowTitle.CANONICAL_ASSET) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val alerts = assetId.flatMapLatest { getPriceAlerts(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val grouped = alerts.map { getPriceAlerts.groupByTargetAndAsset(it) }

    val isAutoAlertEnabled = grouped.map { it[null].orEmpty().isNotEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val sections: StateFlow<List<ListSection<PriceAlertDataAggregate>>> = combine(grouped, assetId) { grouped, assetId ->
        grouped.entries.mapNotNull { (key, items) ->
            val id = key ?: return@mapNotNull null
            if (items.isEmpty()) return@mapNotNull null
            ListSection(
                id = id.toIdentifier(),
                title = if (assetId != null) context.getString(R.string.stake_active) else items.first().title,
                items = items,
            )
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val priceAlertEnabled = assetId.flatMapLatest { id ->
        if (id == null) {
            alertsEnabled
        } else {
            getAssetPriceAlertState.isAssetPriceAlertEnabled(id)
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isRefreshing = refreshState.asStateFlow()

    private val errorState = MutableStateFlow<GemErrorText?>(null)
    val error: StateFlow<GemErrorText?> = errorState.asStateFlow()

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

    fun toggleAutoAlert(enabled: Boolean) = viewModelScope.launch(Dispatchers.IO) {
        val assetId = assetId.value ?: return@launch
        setAutoAlert(assetId, enabled)
    }

    fun excludeAsset(priceAlertId: String) = viewModelScope.launch(Dispatchers.IO) {
        val alert = alerts.value.firstOrNull { it.id == priceAlertId } ?: return@launch
        runCatchingCancellable { service.deletePriceAlerts(listOf(alert.priceAlert.toGem())) }
            .onFailure { errorState.value = it.errorText() }
    }

    fun includeAsset(assetId: AssetId, callback: (Asset) -> Unit) = viewModelScope.launch(Dispatchers.IO) {
        setAutoAlert(assetId, true)

        val assetInfo = getAssetTokenInfo(assetId).firstOrNull() ?: return@launch
        withContext(Dispatchers.Main) { callback(assetInfo.asset) }
    }

    private suspend fun setAutoAlert(assetId: AssetId, enabled: Boolean) {
        runCatchingCancellable { service.setAutoAlert(assetId.toIdentifier(), enabled) }
            .onFailure { errorState.value = it.errorText() }
    }

    fun clearError() = errorState.update { null }

    private companion object {
        const val TAG = "PriceAlerts"
    }

}
