package com.gemwallet.android.features.price_alerts.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.AssetQueryOptional
import com.gemwallet.android.data.services.store.queries.PriceAlertsQuery
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAssetPriceAlerts
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPriceAlertItem
import uniffi.gemstone.GemPriceAlertServiceInterface
import uniffi.gemstone.PriceAlertFormatter
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetPriceAlertsViewModel @Inject constructor(
    priceAlertsQuery: PriceAlertsQuery,
    getCurrentWalletId: GetCurrentWalletId,
    assetQuery: AssetQueryOptional,
    private val service: GemPriceAlertServiceInterface,
    private val priceAlertFormatter: PriceAlertFormatter,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val assetId = savedStateHandle.requireAssetId(RouteArgument.AssetId)

    private val refreshState = MutableStateFlow(false)
    private val loadState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    private val assetInfo = getCurrentWalletId().flatMapLatest { walletId -> assetQuery(walletId.id, assetId) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetAlerts: StateFlow<GemAssetPriceAlerts?> = combine(assetInfo, priceAlertsQuery(assetId), loadState) { info, alerts, state ->
        info ?: return@combine null
        priceAlertFormatter.assetAlerts(info.asset.toGem(), info.price?.toGem(), alerts.map { it.toGem() }, service.getCurrency(), state)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val sections: StateFlow<List<ListSection<GemPriceAlertItem>>> = assetAlerts.map { assetAlerts ->
        assetAlerts?.alerts.orEmpty().takeIf { it.isNotEmpty() }?.let { alerts ->
            listOf(ListSection(id = assetId.toIdentifier(), title = context.getString(R.string.stake_active), items = alerts))
        }.orEmpty()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val isRefreshing = refreshState.asStateFlow()

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

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
        loadState.update { service.refresh(assetId.toIdentifier()) }
    }

    fun toggleAutoAlert(enabled: Boolean) = viewModelScope.launch(ioDispatcher) {
        val asset = assetInfo.value?.asset ?: return@launch
        runCatchingCancellable { service.setAutoAlert(asset.toGem(), enabled) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun excludeAsset(priceAlertId: String) = viewModelScope.launch(ioDispatcher) {
        val alert = assetAlerts.value?.alerts.orEmpty().firstOrNull { it.id == priceAlertId } ?: return@launch
        runCatchingCancellable { service.deletePriceAlerts(listOf(alert.data.priceAlert)) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun clearError() = errorState.update { null }
}
