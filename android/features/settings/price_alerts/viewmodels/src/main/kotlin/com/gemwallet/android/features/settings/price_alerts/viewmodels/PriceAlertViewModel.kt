package com.gemwallet.android.features.settings.price_alerts.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.settings.price_alerts.viewmodels.localization.footer
import com.gemwallet.android.features.settings.price_alerts.viewmodels.localization.title
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PriceAlertData
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
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
import uniffi.gemstone.GemPriceAlertSectionKind
import uniffi.gemstone.GemPriceAlertServiceInterface
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.PriceAlertFormatter
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PriceAlertViewModel @Inject constructor(
    getPriceAlerts: GetPriceAlerts,
    private val getAssetTokenInfo: GetAssetTokenInfo,
    private val service: GemPriceAlertServiceInterface,
    private val priceAlertFormatter: PriceAlertFormatter,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
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
        .mapLatest { it?.toAssetInfoDataAggregate(GemSelectAssetType.PriceAlert.flow().rowStyle) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val alerts = assetId.flatMapLatest { getPriceAlerts(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val grouped = alerts.map { alerts ->
        val byId = alerts.associateBy { it.id }
        priceAlertFormatter.sections(alerts.map { PriceAlertData(asset = it.asset, price = null, priceAlert = it.priceAlert, rankScore = it.rankScore).toGem() })
            .map { section -> section.kind to section.alertIds.mapNotNull { byId[it] } }
    }

    val isAutoAlertEnabled = grouped.map { sections -> sections.any { (kind, _) -> kind is GemPriceAlertSectionKind.Auto } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val sections: StateFlow<List<ListSection<PriceAlertDataAggregate>>> = combine(grouped, assetId) { grouped, assetId ->
        grouped.mapNotNull { (kind, items) ->
            when {
                assetId == null -> ListSection(id = kind.sectionId(), title = kind.title(), items = items, footer = kind.footer(context))
                kind is GemPriceAlertSectionKind.Asset -> ListSection(id = kind.sectionId(), title = context.getString(R.string.stake_active), items = items)
                else -> null
            }
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val priceAlertEnabled = assetId.flatMapLatest { id ->
        if (id == null) alertsEnabled else isAutoAlertEnabled
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isRefreshing = refreshState.asStateFlow()

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    init {
        val initialAssetId = savedStateHandle.get<String?>(RouteArgument.AssetId.key)?.toAssetId()
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.sync(initialAssetId?.toIdentifier()) }
                .onFailure { Log.e(TAG, "price alerts sync failed", it) }
        }
    }

    fun refresh() {
        viewModelScope.launch(ioDispatcher) {
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

    fun togglePriceAlerts(enable: Boolean) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.setEnabled(enable) }
            .onFailure { errorState.value = it.errorText().text(context) }
        alertsEnabled.update { service.isEnabled() }
    }

    fun toggleAutoAlert(enabled: Boolean) = viewModelScope.launch(ioDispatcher) {
        val assetId = assetId.value ?: return@launch
        setAutoAlert(assetId, enabled)
    }

    fun excludeAsset(priceAlertId: String) = viewModelScope.launch(ioDispatcher) {
        val alert = alerts.value.firstOrNull { it.id == priceAlertId } ?: return@launch
        runCatchingCancellable { service.deletePriceAlerts(listOf(alert.priceAlert.toGem())) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun includeAsset(assetId: AssetId, callback: (Asset) -> Unit) = viewModelScope.launch(ioDispatcher) {
        setAutoAlert(assetId, true)

        val assetInfo = getAssetTokenInfo(assetId).firstOrNull() ?: return@launch
        withContext(Dispatchers.Main) { callback(assetInfo.asset) }
    }

    private suspend fun setAutoAlert(assetId: AssetId, enabled: Boolean) {
        runCatchingCancellable { service.setAutoAlert(assetId.toIdentifier(), enabled) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun clearError() = errorState.update { null }

    private companion object {
        const val TAG = "PriceAlerts"
    }
}

private fun GemPriceAlertSectionKind.sectionId(): String = when (this) {
    GemPriceAlertSectionKind.Auto -> "auto"
    is GemPriceAlertSectionKind.Asset -> assetId
}
