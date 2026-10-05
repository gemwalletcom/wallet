package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetWalletHomeState
import com.gemwallet.android.application.preferences.cases.ObservablePreferences
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.assets
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.message
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
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemWalletHomeServiceInterface
import uniffi.gemstone.GemWalletHomeViewState
import javax.inject.Inject

@HiltViewModel
class WalletViewModel @Inject constructor(
    private val service: GemWalletHomeServiceInterface,
    getActiveAssetsInfo: GetActiveAssetsInfo,
    getWalletHomeState: GetWalletHomeState,
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

    val isLoadingAssets = MutableStateFlow(false)

    val isRefreshing = MutableStateFlow(false)

    val homeState = getWalletHomeState.walletHomeState()

    val isBalanceHidden = preferences.isHideBalances()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private val walletAssets = getActiveAssetsInfo.assetsInfo()

    val pinnedAssets = sectionAssets { it.pinnedAssetIds }

    val unpinnedAssets = sectionAssets { it.assetIds }

    private fun sectionAssets(ids: (GemWalletHomeViewState) -> List<String>): StateFlow<List<AssetInfoDataAggregate>> {
        val items = { state: GemWalletHomeViewState?, assets: List<AssetInfoDataAggregate> -> assets.assets(state?.let(ids).orEmpty()) { it.asset.id } }
        return combine(homeState, walletAssets, items)
            .flowOn(ioDispatcher)
            .stateIn(viewModelScope, SharingStarted.Eagerly, items(homeState.value, walletAssets.value))
    }

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
        service.hideAsset(assetId, TAG)
    }

    fun togglePin(assetId: AssetId) = viewModelScope.launch(ioDispatcher) {
        val item = walletAssets.value.firstOrNull { it.id == assetId } ?: return@launch
        runCatchingCancellable { service.setAssetPinned(item.asset.toGem(), !item.pinned) }
            .onSuccess { emitToast(it.message(context)) }
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
