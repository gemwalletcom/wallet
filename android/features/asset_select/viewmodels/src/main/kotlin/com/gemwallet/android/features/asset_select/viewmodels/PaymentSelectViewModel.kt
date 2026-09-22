package com.gemwallet.android.features.asset_select.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.features.asset_select.viewmodels.models.BaseSelectSearch
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetType
import javax.inject.Inject

@HiltViewModel
class PaymentSelectViewModel @Inject constructor(
    getSession: GetSession,
    searchService: AssetsSearchService,
    recentAssetsService: RecentAssetsService,
    service: GemAssetSelectionServiceInterface,
    @IoDispatcher ioDispatcher: CoroutineDispatcher,
    @ApplicationContext context: Context,
    savedStateHandle: SavedStateHandle,
) : BaseAssetSelectViewModel(
    getSession,
    recentAssetsService,
    service,
    BaseSelectSearch(searchService),
    GemSelectAssetType.Payment(savedStateHandle.paymentAssetIds()),
    ioDispatcher,
    context,
)
