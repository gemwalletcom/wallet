package com.gemwallet.android.features.settings.price_alerts.viewmodels

import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.asset_select.coordinators.ToggleAssetPin
import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlerts
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectSearch
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@HiltViewModel
class PriceAlertsSelectViewModel @Inject constructor(
    getPriceAlerts: GetPriceAlerts,
    getSession: GetSession,
    getRecentAssets: GetRecentAssets,
    switchAssetVisibility: SwitchAssetVisibility,
    toggleAssetPin: ToggleAssetPin,
    assetsRepository: AssetsRepository,
    searchTokensCase: SearchTokensCase,
) : BaseAssetSelectViewModel(
    getSession,
    getRecentAssets,
    switchAssetVisibility,
    toggleAssetPin,
    searchTokensCase,
    PriceAlertSelectSearch(assetsRepository, getPriceAlerts),
) {
    override fun getRecentType(): RecentType? = null
}

@OptIn(ExperimentalCoroutinesApi::class)
open class PriceAlertSelectSearch(
    private val assetsRepository: AssetsRepository,
    getPriceAlerts: GetPriceAlerts,
) : SelectSearch {

    val addedPriceAlerts = getPriceAlerts().map { items -> items.map { it.assetId } }

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return combine(filters, addedPriceAlerts) { filters, alerts -> Pair(filters,alerts) }
            .flatMapLatest {
                val (filters, alerts) = it
                assetsRepository.search(filters?.query ?: "", filters?.tag?.let { listOf(it) } ?: emptyList(), true)
            }
            .map { items -> items.distinctBy { it.asset.id.toIdentifier() } }
            .flowOn(Dispatchers.IO)
    }
}
