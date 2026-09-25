package com.gemwallet.android.features.asset_select.viewmodels

import android.content.Context
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import dagger.assisted.Assisted
import dagger.assisted.AssistedFactory
import dagger.assisted.AssistedInject
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetType

@HiltViewModel(assistedFactory = AssetSelectViewModel.Factory::class)
class AssetSelectViewModel @AssistedInject constructor(
    @Assisted selectType: GemSelectAssetType,
    getSession: GetSession,
    searchService: AssetsSearchService,
    recentActivityQuery: RecentActivityQuery,
    service: GemAssetSelectionServiceInterface,
    @IoDispatcher ioDispatcher: CoroutineDispatcher,
    @ApplicationContext context: Context,
) : BaseAssetSelectViewModel(
    getSession,
    recentActivityQuery,
    service,
    BaseSelectSearch(searchService),
    selectType,
    ioDispatcher,
    context,
) {
    @AssistedFactory
    interface Factory {
        fun create(selectType: GemSelectAssetType): AssetSelectViewModel
    }
}
