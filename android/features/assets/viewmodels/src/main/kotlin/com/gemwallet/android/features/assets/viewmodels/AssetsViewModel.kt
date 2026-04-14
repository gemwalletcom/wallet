package com.gemwallet.android.features.assets.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.coordinators.GetHideBalancesState
import com.gemwallet.android.application.assets.coordinators.GetImportInProgress
import com.gemwallet.android.application.assets.coordinators.GetShowWelcomeBanner
import com.gemwallet.android.application.assets.coordinators.GetWalletSummary
import com.gemwallet.android.application.assets.coordinators.HideAsset
import com.gemwallet.android.application.assets.coordinators.HideWelcomeBanner
import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.application.assets.coordinators.ToggleAssetPin
import com.gemwallet.android.application.assets.coordinators.ToggleHideBalances
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class AssetsViewModel @Inject constructor(
    private val syncAssets: SyncAssets,
    private val hideAsset: HideAsset,
    private val toggleAssetPin: ToggleAssetPin,
    private val toggleHideBalances: ToggleHideBalances,
    private val hideWelcomeBanner: HideWelcomeBanner,
    getImportInProgress: GetImportInProgress,
    getActiveAssetsInfo: GetActiveAssetsInfo,
    getWalletSummary: GetWalletSummary,
    getHideBalancesState: GetHideBalancesState,
    getShowWelcomeBanner: GetShowWelcomeBanner,
) : ViewModel() {

    private data class AssetGroups(
        val pinned: List<AssetInfoDataAggregate> = emptyList(),
        val unpinned: List<AssetInfoDataAggregate> = emptyList(),
    )

    val importInProgress = getImportInProgress()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val isRefreshing = MutableStateFlow(false)

    private val isHideBalances = getHideBalancesState()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private val assetGroups = getActiveAssetsInfo.getAssetsInfo(isHideBalances)
        .map { items ->
            val (pinned, unpinned) = items.partition { it.pinned }
            AssetGroups(
                pinned = pinned,
                unpinned = unpinned,
            )
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, AssetGroups())

    private val isWalletEmpty = assetGroups
        .map { groups ->
            groups.pinned.all { it.isZeroBalance }
                && groups.unpinned.all { it.isZeroBalance }
        }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val pinnedAssets = assetGroups
        .map { it.pinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val unpinnedAssets = assetGroups
        .map { it.unpinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val walletSummary = getWalletSummary.getWalletSummary()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val showWelcomeBanner = getShowWelcomeBanner(isWalletEmpty)
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    fun onRefresh() = viewModelScope.launch(Dispatchers.IO) {
        isRefreshing.value = true
        try {
            syncAssets()
        } finally {
            isRefreshing.value = false
        }
    }

    fun hideAsset(assetId: AssetId) = viewModelScope.launch {
        hideAsset.invoke(assetId)
    }

    fun togglePin(assetId: AssetId) = viewModelScope.launch {
        toggleAssetPin(assetId)
    }

    fun hideBalances() = viewModelScope.launch {
        toggleHideBalances()
    }

    fun onHideWelcomeBanner() = viewModelScope.launch {
        hideWelcomeBanner()
    }
}
