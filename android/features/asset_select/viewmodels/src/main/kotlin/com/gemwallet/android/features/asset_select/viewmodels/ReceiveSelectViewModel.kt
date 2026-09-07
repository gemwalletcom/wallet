package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetType
import javax.inject.Inject

@HiltViewModel
class ReceiveSelectViewModel @Inject constructor(
    getSession: GetSession,
    searchService: AssetsSearchService,
    recentAssetsService: RecentAssetsService,
    service: GemAssetSelectionServiceInterface,
) : BaseAssetSelectViewModel(
    getSession,
    recentAssetsService,
    service,
    BaseSelectSearch(searchService),
    GemSelectAssetType.RECEIVE,
)
