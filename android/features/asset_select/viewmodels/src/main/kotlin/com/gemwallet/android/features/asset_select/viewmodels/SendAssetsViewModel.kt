package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.GetSelectAssetsInfo
import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.asset_select.coordinators.ToggleAssetPin
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.wallet.core.primitives.AssetTag
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
    switchAssetVisibility: SwitchAssetVisibility,
    toggleAssetPin: ToggleAssetPin,
    searchTokensCase: SearchTokensCase,
) : BaseAssetSelectViewModel(
    getSession,
    getRecentAssets,
    switchAssetVisibility,
    toggleAssetPin,
    searchTokensCase,
    SendSelectSearch(searchSelectAssets, getSelectAssetsInfo),
) {
    override fun getRecentType(): RecentType? = RecentType.Send

    override fun getTags(): List<AssetTag?> = listOf(
        null,
        AssetTag.Stablecoins,
    )
}

@OptIn(ExperimentalCoroutinesApi::class)
class SendSelectSearch(
    private val searchSelectAssets: SearchSelectAssets,
    private val getSelectAssetsInfo: GetSelectAssetsInfo,
) : BaseSelectSearch(searchSelectAssets) {
    override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> {
        return filters
            .map { filters -> filters?.query.orEmpty() to filters?.tag }
            .flatMapLatest { (query, tag) ->
                val source = if (query.isEmpty() && tag == null) {
                    getSelectAssetsInfo()
                } else {
                    searchSelectAssets(query, tag?.let(::listOf) ?: emptyList())
                }

                source.map { items ->
                    filter(items).distinctBy { it.asset.id.toIdentifier() }
                }
            }
    }

    override fun filter(items: List<AssetInfo>): List<AssetInfo> = items.filter { it.balance.totalAmount != 0.0 }
}
