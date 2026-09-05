package com.gemwallet.android.features.settings.price_alerts.viewmodels

import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetType
import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.features.asset_select.viewmodels.BaseAssetSelectViewModel
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectSearch
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import javax.inject.Inject

@HiltViewModel
class PriceAlertsSelectViewModel @Inject constructor(
    getSession: GetSession,
    getRecentAssets: GetRecentAssets,
    searchService: AssetsSearchService,
    service: GemAssetSelectionServiceInterface,
) : BaseAssetSelectViewModel(
    getSession,
    getRecentAssets,
    service,
    PriceAlertSelectSearch(searchService),
    GemSelectAssetType.PRICE_ALERT,
)

@OptIn(ExperimentalCoroutinesApi::class)
open class PriceAlertSelectSearch(
    private val searchService: AssetsSearchService,
) : SelectSearch {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return filters
            .flatMapLatest { filters ->
                searchService.search(
                    filters?.query ?: "",
                    true
                )
            }
            .flowOn(Dispatchers.IO)
    }
}
