package com.gemwallet.android.features.assets.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetWalletSummary
import com.gemwallet.android.application.preferences.cases.ObservablePreferences
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.assetSections
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.banner.BannerRowUIModel
import com.gemwallet.android.ui.components.banner.uiModel
import com.gemwallet.android.ui.components.screen.assetPinnedToast
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ToastEmitter
import com.gemwallet.android.ui.models.ToastEmitterImpl
import com.gemwallet.android.ui.models.ToastMessage
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemWalletHomeServiceInterface
import javax.inject.Inject

@HiltViewModel
class AssetsViewModel @Inject constructor(
    private val service: GemWalletHomeServiceInterface,
    getActiveAssetsInfo: GetActiveAssetsInfo,
    getWalletSummary: GetWalletSummary,
    private val getSession: GetSession,
    private val preferences: ObservablePreferences,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel(),
    ToastEmitter by ToastEmitterImpl() {

    val currentWalletId = getSession()
        .map { it?.wallet?.id }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private fun groups(items: List<AssetInfoDataAggregate>) = items.assetSections(assetId = { it.asset.id }, isPinned = { it.pinned })

    val isLoadingAssets = MutableStateFlow(false)

    val isRefreshing = MutableStateFlow(false)

    private val assetGroups = getActiveAssetsInfo.assetsInfo()
        .map(::groups)
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, groups(getActiveAssetsInfo.assetsInfo().value))

    val pinnedAssets = assetGroups
        .map { it.pinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, assetGroups.value.pinned)

    val unpinnedAssets = assetGroups
        .map { it.unpinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, assetGroups.value.unpinned)

    val walletSummary = getWalletSummary.getWalletSummary()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val bannerRow: StateFlow<BannerRowUIModel?> = walletSummary.map { summary -> summary?.state?.banner?.uiModel(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val collectionsAvailable = walletSummary
        .map { it?.state?.showCollections ?: false }
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    init {
        viewModelScope.launch(ioDispatcher) {
            currentWalletId.filterNotNull().collectLatest { loadOnce() }
        }
    }

    fun onRefresh() = viewModelScope.launch(ioDispatcher) {
        isRefreshing.value = true
        try {
            refresh()
        } finally {
            isRefreshing.value = false
        }
    }

    private suspend fun loadOnce() {
        isLoadingAssets.value = service.showsInitialLoading()
        refresh()
    }

    private suspend fun refresh() {
        runCatchingCancellable { service.refresh() }
            .onFailure { Log.e(TAG, "assets refresh failed", it) }
        isLoadingAssets.value = service.showsInitialLoading()
    }

    fun hideAsset(assetId: AssetId) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.setAssetsEnabled(listOf(assetId.toIdentifier()), false) }
            .onFailure { Log.e(TAG, "hiding ${assetId.toIdentifier()} failed", it) }
    }

    fun togglePin(assetId: AssetId) = viewModelScope.launch(ioDispatcher) {
        val item = assetGroups.value.let { it.pinned + it.unpinned }.firstOrNull { it.id == assetId } ?: return@launch
        runCatchingCancellable { service.setAssetPinned(assetId.toIdentifier(), !item.pinned) }
            .onSuccess { emitToast(assetPinnedToast(context, item.asset.name, !item.pinned)) }
            .onFailure { Log.e(TAG, "pinning ${assetId.toIdentifier()} failed", it) }
    }

    fun hideBalances() {
        preferences.hideBalances()
    }

    fun closeBanner(key: GemBannerKey) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.closeBanner(key) }
            .onFailure { emitToast(ToastMessage(it.errorText().text(context), R.drawable.ic_error)) }
    }

    private companion object {
        const val TAG = "Assets"
    }
}
