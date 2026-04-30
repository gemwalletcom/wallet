package com.gemwallet.android.features.settings.price_alerts.viewmodels

import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.asset_select.coordinators.ToggleAssetPin
import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.assets.AssetsRepository
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
    updateRecentAsset: UpdateRecentAsset,
    switchAssetVisibility: SwitchAssetVisibility,
    toggleAssetPin: ToggleAssetPin,
    assetsRepository: AssetsRepository,
    searchTokensCase: SearchTokensCase,
) : BaseAssetSelectViewModel(
    getSession,
    getRecentAssets,
    updateRecentAsset,
    switchAssetVisibility,
    toggleAssetPin,
    searchTokensCase,
    PriceAlertSelectSearch(assetsRepository),
) {
    override val showRecents: Boolean get() = false
}

@OptIn(ExperimentalCoroutinesApi::class)
open class PriceAlertSelectSearch(
    private val assetsRepository: AssetsRepository,
) : SelectSearch {

    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return filters
            .flatMapLatest { filters ->
                assetsRepository.search(
                    filters?.query ?: "",
                    filters?.tag?.let { listOf(it) } ?: emptyList(),
                    true
                )
            }
            .flowOn(Dispatchers.IO)
    }
}
