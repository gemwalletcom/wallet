package com.gemwallet.android.features.assets.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.coordinators.GetHideBalancesState
import com.gemwallet.android.application.assets.coordinators.HideAsset
import com.gemwallet.android.application.assets.coordinators.ToggleAssetPin
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class HiddenAssetsViewModel @Inject constructor(
    private val hideAsset: HideAsset,
    private val toggleAssetPin: ToggleAssetPin,
    getActiveAssetsInfo: GetActiveAssetsInfo,
    getHideBalancesState: GetHideBalancesState,
) : ViewModel() {

    val hiddenAssets = getHideBalancesState()
        .flatMapLatest { hideBalance -> getActiveAssetsInfo.getHiddenAssetsInfo(hideBalance) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun hideAsset(assetId: AssetId) = viewModelScope.launch {
        hideAsset.invoke(assetId)
    }

    fun togglePin(assetId: AssetId) = viewModelScope.launch {
        toggleAssetPin(assetId)
    }
}
