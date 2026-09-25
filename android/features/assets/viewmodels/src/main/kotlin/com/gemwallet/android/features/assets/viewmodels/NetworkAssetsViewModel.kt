package com.gemwallet.android.features.assets.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregates
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.assetAddedToast
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ToastEmitter
import com.gemwallet.android.ui.models.ToastEmitterImpl
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.navigation.requireChain
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemNetworkAssetSections
import uniffi.gemstone.GemWalletHomeServiceInterface
import uniffi.gemstone.networkAssetSections
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class NetworkAssetsViewModel @Inject constructor(
    assetStore: GemstoneAssetStore,
    getCurrentWalletId: GetCurrentWalletId,
    private val service: GemWalletHomeServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel(),
    ToastEmitter by ToastEmitterImpl() {

    private val chain: Chain = savedStateHandle.requireChain()

    val title: String = context.getString(R.string.assets_title)

    private val assetGroups: StateFlow<NetworkAssetGroups> = getCurrentWalletId()
        .flatMapLatest { walletId ->
            combine(
                assetStore.observeAssetsInfoByChain(walletId.id, chain).flowOn(ioDispatcher),
                assetStore.observeHiddenAssetsInfoByChain(walletId.id, chain).flowOn(ioDispatcher),
            ) { active, hidden -> groups(active, hidden) }
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, NetworkAssetGroups())

    val pinned: StateFlow<List<AssetInfoDataAggregate>> = assetGroups
        .map { it.pinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, assetGroups.value.pinned)

    val unpinned: StateFlow<List<AssetInfoDataAggregate>> = assetGroups
        .map { it.unpinned }
        .stateIn(viewModelScope, SharingStarted.Eagerly, assetGroups.value.unpinned)

    val hidden: StateFlow<List<AssetInfoDataAggregate>> = assetGroups
        .map { it.hidden }
        .stateIn(viewModelScope, SharingStarted.Eagerly, assetGroups.value.hidden)

    val sections: StateFlow<GemNetworkAssetSections> = assetGroups
        .map { it.sections }
        .stateIn(viewModelScope, SharingStarted.Eagerly, assetGroups.value.sections)

    init {
        viewModelScope.launch(ioDispatcher) {
            val loaded = assetGroups.first { it.isLoaded }
            runCatchingCancellable { service.updateBalances(loaded.assetIds()) }
                .onFailure { Log.e(TAG, "balances update failed for ${chain.string}", it) }
        }
    }

    private fun groups(active: List<AssetInfo>, hidden: List<AssetInfo>): NetworkAssetGroups {
        val ids = networkAssetSections(
            active = active.map { it.asset.id.toIdentifier() },
            pinned = active.filter { it.metadata.isPinned }.map { it.asset.id.toIdentifier() },
            hidden = hidden.map { it.asset.id.toIdentifier() },
        )
        val byId = (active + hidden).associateBy { it.asset.id.toIdentifier() }
        val assets = { assetIds: List<String> -> assetIds.mapNotNull(byId::get).toAssetInfoDataAggregates() }
        return NetworkAssetGroups(
            pinned = assets(ids.pinned),
            unpinned = assets(ids.unpinned),
            hidden = assets(ids.hidden),
            sections = ids.sections,
            isLoaded = true,
        )
    }

    fun hideAsset(assetId: AssetId) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.setAssetsEnabled(listOf(assetId.toIdentifier()), false) }
            .onFailure { Log.e(TAG, "hiding ${assetId.toIdentifier()} failed", it) }
    }

    fun addToWallet(assetId: AssetId) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.setAssetsEnabled(listOf(assetId.toIdentifier()), true) }
            .onSuccess { emitToast(assetAddedToast(context)) }
            .onFailure { emitToast(ToastMessage(it.errorText().text(context), R.drawable.ic_error)) }
    }

    fun togglePin(assetId: AssetId) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.setAssetPinned(assetId.toIdentifier(), assetGroups.value.pinned.none { it.id == assetId }) }
            .onFailure { Log.e(TAG, "pinning ${assetId.toIdentifier()} failed", it) }
    }

    private companion object {
        const val TAG = "NetworkAssets"
    }
}

private data class NetworkAssetGroups(
    val pinned: List<AssetInfoDataAggregate> = emptyList(),
    val unpinned: List<AssetInfoDataAggregate> = emptyList(),
    val hidden: List<AssetInfoDataAggregate> = emptyList(),
    val sections: GemNetworkAssetSections = GemNetworkAssetSections(showsPinned = false, showsUnpinned = false, showsHidden = false, showsEmpty = true),
    val isLoaded: Boolean = false,
) {
    fun assetIds(): List<String> = (pinned + unpinned + hidden).map { it.id.toIdentifier() }
}
