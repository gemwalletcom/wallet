package com.gemwallet.android.features.price_alerts.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetQueryOptional
import com.gemwallet.android.data.services.store.queries.PriceAlertsQuery
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.localization.footer
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.title
import com.gemwallet.android.ui.models.ListSection
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPriceAlertItem
import uniffi.gemstone.GemPriceAlertList
import uniffi.gemstone.GemPriceAlertSectionKind
import uniffi.gemstone.GemPriceAlertServiceInterface
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
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val refreshState = MutableStateFlow(false)
    private val alertsEnabled = MutableStateFlow(service.isEnabled())
    private val loadState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    private val list: StateFlow<GemPriceAlertList?> = combine(priceAlertsQuery(), loadState) { alerts, state ->
        priceAlertFormatter.list(alerts.map { it.toGem() }, service.getCurrency(), state)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val sections: StateFlow<List<ListSection<GemPriceAlertItem>>> = list.map { list ->
        list?.sections.orEmpty().map { section -> ListSection(id = section.kind.sectionId(), title = section.kind.title(), items = section.items, footer = section.kind.footer(context)) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val phase: StateFlow<GemListPhase?> = list.map { it?.phase }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val priceAlertEnabled = alertsEnabled.asStateFlow()

    val isRefreshing = refreshState.asStateFlow()

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()
    val toast = MutableStateFlow<GemToast?>(null)

    init {
        viewModelScope.launch(ioDispatcher) { sync() }
    }

    fun refresh() {
        viewModelScope.launch(ioDispatcher) {
            try {
                refreshState.value = true
                sync()
            } finally {
                refreshState.value = false
            }
        }
    }

    private suspend fun sync() {
        loadState.update { service.refresh(null) }
    }

    fun togglePriceAlerts(enable: Boolean) = viewModelScope.launch(ioDispatcher) {
        alertsEnabled.value = enable
        runCatchingCancellable { service.setEnabled(enable) }
            .onFailure { errorState.value = it.errorText().text(context) }
        alertsEnabled.update { service.isEnabled() }
    }

    fun excludeAsset(priceAlertId: String) = viewModelScope.launch(ioDispatcher) {
        val alert = list.value?.sections.orEmpty().flatMap { it.items }.firstOrNull { it.id == priceAlertId } ?: return@launch
        runCatchingCancellable { service.deletePriceAlerts(listOf(alert.data.priceAlert)) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun includeAsset(assetId: AssetId) = viewModelScope.launch(ioDispatcher) {
        val asset = getCurrentWalletId().flatMapLatest { walletId -> assetQuery(walletId.id, assetId) }.firstOrNull()?.asset ?: return@launch
        runCatchingCancellable { service.setAutoAlert(asset.toGem(), true) }
            .onSuccess { toast.value = it }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun clearError() = errorState.update { null }
}

private fun GemPriceAlertSectionKind.sectionId(): String = when (this) {
    GemPriceAlertSectionKind.Auto -> "auto"
    is GemPriceAlertSectionKind.Asset -> assetId
}
