package com.gemwallet.android.features.asset_select.viewmodels

import uniffi.gemstone.GemAssetSelectionServiceInterface
import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.application.asset_select.cases.SearchSelectAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetSelectViewModel @Inject constructor(
    getSession: GetSession,
    searchSelectAssets: SearchSelectAssets,
    getRecentAssets: GetRecentAssets,
    service: GemAssetSelectionServiceInterface,
) : BaseAssetSelectViewModel(
    getSession,
    getRecentAssets,
    service,
    BaseSelectSearch(searchSelectAssets),
)
