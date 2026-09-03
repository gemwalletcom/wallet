package com.gemwallet.android.features.assets.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetHideBalancesState
import com.gemwallet.android.application.assets.cases.GetShowWelcomeBanner
import com.gemwallet.android.ext.onboardingBannerKey
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.models.AssetToast
import com.gemwallet.android.ui.models.AssetToastEmitter
import com.gemwallet.android.ui.models.AssetToastEmitterImpl
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemBannerAction
import uniffi.gemstone.GemWalletHomeServiceInterface
import javax.inject.Inject

@HiltViewModel
class AssetsViewModel @Inject constructor(
    private val service: GemWalletHomeServiceInterface,
    getActiveAssetsInfo: GetActiveAssetsInfo,
    getWalletSummary: GetWalletSummary,
    getHideBalancesState: GetHideBalancesState,
    getShowWelcomeBanner: GetShowWelcomeBanner,
    private val getSession: GetSession,
    private val userConfig: UserConfig,
) : ViewModel(), AssetToastEmitter by AssetToastEmitterImpl() {

    val currentWalletId = getSession()
        .map { it?.wallet?.id }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val collectionsAvailable = getSession()
        .map { it?.wallet?.let(userConfig::showCollections) ?: false }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private data class AssetGroups(
        val pinned: List<AssetInfoDataAggregate> = emptyList(),
        val unpinned: List<AssetInfoDataAggregate> = emptyList(),
    )

    val isLoadingAssets = MutableStateFlow(false)

    val isRefreshing = MutableStateFlow(false)

    private val isHideBalances = getHideBalancesState()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private val assetGroups = getActiveAssetsInfo.getAssetsInfo(isHideBalances)
        .map { items ->
            val (pinned, unpinned) = items.partition { it.pinned }
            AssetGroups(pinned = pinned, unpinned = unpinned)
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, AssetGroups())

    val pinnedAssets = assetGroups
        .map { it.pinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val unpinnedAssets = assetGroups
        .map { it.unpinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val walletSummary = getWalletSummary.getWalletSummary()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val showWelcomeBanner = getShowWelcomeBanner()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    init {
        viewModelScope.launch(Dispatchers.IO) {
            currentWalletId.filterNotNull().collectLatest { loadOnce() }
        }
    }

    fun onRefresh() = viewModelScope.launch(Dispatchers.IO) {
        isRefreshing.value = true
        try {
            refresh()
        } finally {
            isRefreshing.value = false
        }
    }

    private suspend fun loadOnce() {
        val showsLoading = runCatchingCancellable { service.showsInitialLoading() }.getOrDefault(false)
        if (showsLoading) isLoadingAssets.value = true
        try {
            refresh()
        } finally {
            if (showsLoading) isLoadingAssets.value = false
        }
    }

    private suspend fun refresh() {
        val assetIds = assetGroups.value.let { it.pinned + it.unpinned }.map { it.id.toIdentifier() }
        runCatchingCancellable { service.refresh(assetIds) }
            .onFailure { Log.e(TAG, "assets refresh failed", it) }
    }

    fun hideAsset(assetId: AssetId) = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { service.setAssetsEnabled(listOf(assetId.toIdentifier()), false) }
            .onFailure { Log.e(TAG, "hiding ${assetId.toIdentifier()} failed", it) }
    }

    fun togglePin(assetId: AssetId) = viewModelScope.launch(Dispatchers.IO) {
        val item = assetGroups.value.let { it.pinned + it.unpinned }.firstOrNull { it.id == assetId } ?: return@launch
        runCatchingCancellable { service.setAssetPinned(assetId.toIdentifier(), !item.pinned) }
            .onFailure { Log.e(TAG, "pinning ${assetId.toIdentifier()} failed", it) }
        emitToast(AssetToast.Pin(item.asset.name, !item.pinned))
    }

    fun hideBalances() {
        userConfig.hideBalances()
    }

    fun onHideWelcomeBanner() = viewModelScope.launch(Dispatchers.IO) {
        val wallet = getSession().value?.wallet ?: return@launch
        runCatchingCancellable { service.applyBannerAction(wallet.onboardingBannerKey(), GemBannerAction.Close) }
            .onFailure { Log.e(TAG, "closing the welcome banner failed", it) }
    }

    private companion object {
        const val TAG = "Assets"
    }
}
