package com.gemwallet.android.features.asset_select.viewmodels

import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetType
import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.application.asset_select.cases.GetSelectAssetsInfo
import com.gemwallet.android.application.asset_select.cases.SearchSelectAssets
import uniffi.gemstone.GemAssetAction
import com.gemwallet.android.domains.asset.eligible
import com.gemwallet.android.domains.asset.queryFilters
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
open class SendSelectViewModel @Inject constructor(
    getSession: GetSession,
    searchSelectAssets: SearchSelectAssets,
    getSelectAssetsInfo: GetSelectAssetsInfo,
    getRecentAssets: GetRecentAssets,
    service: GemAssetSelectionServiceInterface,
) : BaseAssetSelectViewModel(
    getSession,
    getRecentAssets,
    service,
    SendSelectSearch(searchSelectAssets, getSelectAssetsInfo),
    GemSelectAssetType.SEND,
)

@OptIn(ExperimentalCoroutinesApi::class)
class SendSelectSearch(
    private val searchSelectAssets: SearchSelectAssets,
    private val getSelectAssetsInfo: GetSelectAssetsInfo,
) : BaseSelectSearch(searchSelectAssets) {
    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return filters
            .map { filters -> filters?.query.orEmpty() }
            .flatMapLatest { query ->
                val source = if (query.isEmpty()) {
                    getSelectAssetsInfo()
                } else {
                    searchSelectAssets(query, filters = GemAssetAction.SEND.queryFilters())
                }

                source.map(::filter)
            }
    }

    override fun filter(items: List<AssetInfo>): List<AssetInfo> = GemAssetAction.SEND.eligible(items)
}
