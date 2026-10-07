package com.gemwallet.android.features.price_alerts.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetQueryOptional
import com.gemwallet.android.data.services.store.queries.PriceAlertsQuery
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.footer
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.title
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
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
import uniffi.gemstone.GemAssetPriceAlerts
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPriceAlertItem
import uniffi.gemstone.GemPriceAlertList
import uniffi.gemstone.GemPriceAlertSectionKind
import uniffi.gemstone.GemPriceAlertServiceInterface
import uniffi.gemstone.GemPriceAlertToggle
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.GemToast
import uniffi.gemstone.PriceAlertFormatter
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class PriceAlertsViewModel @Inject constructor(
    priceAlertsQuery: PriceAlertsQuery,
    private val getCurrentWalletId: GetCurrentWalletId,
    private val assetQuery: AssetQueryOptional,
    private val service: GemPriceAlertServiceInterface,
    private val priceAlertFormatter: PriceAlertFormatter,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val refreshState = MutableStateFlow(false)
    private val alertsEnabled = MutableStateFlow(service.isEnabled())
    private val loadState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    val assetId = savedStateHandle.getStateFlow<String?>(RouteArgument.AssetId.key, null)
        .mapLatest { it?.toAssetId() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetInfo = assetId.flatMapLatest { id ->
        if (id != null) assetInfo(id) else flowOf(null)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val asset = assetInfo
        .mapLatest { it?.toAssetInfoDataAggregate(service.getCurrency().toPrimitives(), GemSelectAssetType.PriceAlert.flow().rowStyle) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val alerts = assetId.flatMapLatest { priceAlertsQuery(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val assetAlerts: StateFlow<GemAssetPriceAlerts?> = combine(assetInfo, alerts, loadState) { info, alerts, state ->
        info ?: return@combine null
        priceAlertFormatter.assetAlerts(info.asset.toGem(), info.price?.toGem(), alerts.map { it.toGem() }, service.getCurrency(), state)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val list: StateFlow<GemPriceAlertList?> = combine(alerts, loadState) { alerts, state ->
        priceAlertFormatter.list(alerts.map { it.toGem() }, service.getCurrency(), state)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val sections: StateFlow<List<ListSection<GemPriceAlertItem>>> = combine(list, assetAlerts, assetId) { list, assetAlerts, assetId ->
        when (assetId) {
            null -> list?.sections.orEmpty().map { section -> ListSection(id = section.kind.sectionId(), title = section.kind.title(), items = section.items, footer = section.kind.footer(context)) }

            else -> assetAlerts?.alerts.orEmpty().takeIf { it.isNotEmpty() }?.let { alerts ->
                listOf(ListSection(id = assetId.toIdentifier(), title = context.getString(R.string.stake_active), items = alerts))
            }.orEmpty()
        }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val priceAlertEnabled = assetId.flatMapLatest { id ->
        if (id == null) alertsEnabled else assetAlerts.map { it?.autoAlert == GemPriceAlertToggle.ENABLED }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isRefreshing = refreshState.asStateFlow()

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    val phase: StateFlow<GemListPhase?> = combine(assetId, list, assetAlerts) { assetId, list, assetAlerts ->
        if (assetId == null) list?.phase else assetAlerts?.phase
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    init {
        val initialAssetId = savedStateHandle.get<String?>(RouteArgument.AssetId.key)?.toAssetId()
        viewModelScope.launch(ioDispatcher) {
            sync(initialAssetId?.toIdentifier())
        }
    }

    fun refresh() {
        viewModelScope.launch(ioDispatcher) {
            try {
                refreshState.value = true
                sync(assetId.value?.toIdentifier())
            } finally {
                refreshState.value = false
            }
        }
    }

    private suspend fun sync(assetId: String?) {
        loadState.update { service.refresh(assetId) }
    }

    fun isAssetManage(): Boolean = assetId.value != null

    fun togglePriceAlerts(enable: Boolean) = viewModelScope.launch(ioDispatcher) {
        alertsEnabled.value = enable
        runCatchingCancellable { service.setEnabled(enable) }
            .onFailure { errorState.value = it.errorText().text(context) }
        alertsEnabled.update { service.isEnabled() }
    }

    fun toggleAutoAlert(enabled: Boolean) = viewModelScope.launch(ioDispatcher) {
        val asset = assetInfo.value?.asset ?: return@launch
        setAutoAlert(asset, enabled)
    }

    fun excludeAsset(priceAlertId: String) = viewModelScope.launch(ioDispatcher) {
        val alert = list.value?.sections.orEmpty().flatMap { it.items }.firstOrNull { it.id == priceAlertId } ?: return@launch
        runCatchingCancellable { service.deletePriceAlerts(listOf(alert.data.priceAlert)) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun includeAsset(assetId: AssetId, callback: (GemToast) -> Unit) = viewModelScope.launch(ioDispatcher) {
        val asset = assetInfo(assetId).firstOrNull()?.asset ?: return@launch
        val toast = setAutoAlert(asset, true) ?: return@launch
        withContext(Dispatchers.Main) { callback(toast) }
    }

    private fun assetInfo(assetId: AssetId) = getCurrentWalletId().flatMapLatest { walletId -> assetQuery(walletId.id, assetId) }

    private suspend fun setAutoAlert(asset: Asset, enabled: Boolean): GemToast? = runCatchingCancellable { service.setAutoAlert(asset.toGem(), enabled) }
        .onFailure { errorState.value = it.errorText().text(context) }
        .getOrNull()

    fun clearError() = errorState.update { null }

    private companion object {
        const val TAG = "PriceAlerts"
    }
}

private fun GemPriceAlertSectionKind.sectionId(): String = when (this) {
    GemPriceAlertSectionKind.Auto -> "auto"
    is GemPriceAlertSectionKind.Asset -> assetId
}
