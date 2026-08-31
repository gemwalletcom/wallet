package com.gemwallet.android.features.assets.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetHideBalancesState
import com.gemwallet.android.application.assets.cases.GetImportInProgress
import com.gemwallet.android.application.assets.cases.GetShowWelcomeBanner
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.assets.cases.HideAsset
import com.gemwallet.android.application.assets.cases.HideWelcomeBanner
import com.gemwallet.android.application.assets.cases.SyncAssets
import com.gemwallet.android.application.assets.cases.SetAssetPinned
import com.gemwallet.android.application.assets.cases.ToggleHideBalances
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.ui.models.AssetToast
import com.gemwallet.android.ui.models.AssetToastEmitter
import com.gemwallet.android.ui.models.AssetToastEmitterImpl
import com.gemwallet.android.ext.isNftSupported
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
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
    private val setAssetPinned: SetAssetPinned,
    private val toggleHideBalances: ToggleHideBalances,
    private val hideWelcomeBanner: HideWelcomeBanner,
    getImportInProgress: GetImportInProgress,
    getActiveAssetsInfo: GetActiveAssetsInfo,
    getWalletSummary: GetWalletSummary,
    getHideBalancesState: GetHideBalancesState,
    getShowWelcomeBanner: GetShowWelcomeBanner,
    getSession: GetSession,
) : ViewModel(), AssetToastEmitter by AssetToastEmitterImpl() {

    val currentWalletId = getSession()
        .map { it?.wallet?.id }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val collectionsAvailable = getSession()
        .map { it?.wallet?.isCollectionsAvailable() ?: false }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

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
        val item = assetGroups.value.let { it.pinned + it.unpinned }.firstOrNull { it.id == assetId } ?: return@launch
        setAssetPinned(assetId, !item.pinned)
        emitToast(AssetToast.Pin(item.asset.name, !item.pinned))
    }

    fun hideBalances() = viewModelScope.launch {
        toggleHideBalances()
    }

    fun onHideWelcomeBanner() = viewModelScope.launch {
        hideWelcomeBanner()
    }
}

private fun Wallet.isCollectionsAvailable(): Boolean = when (type) {
    WalletType.Multicoin -> true
    WalletType.Single,
    WalletType.PrivateKey,
    WalletType.View -> accounts.firstOrNull()?.chain?.isNftSupported() ?: false
}
