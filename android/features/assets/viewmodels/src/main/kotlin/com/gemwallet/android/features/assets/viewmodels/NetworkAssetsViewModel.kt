package com.gemwallet.android.features.assets.viewmodels

import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import com.gemwallet.android.domains.asset.assetConfig
import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import android.util.Log
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.model.AssetInfo
import kotlinx.coroutines.CoroutineDispatcher
import uniffi.gemstone.GemAssetRowStyle
import uniffi.gemstone.GemNetworkAssetCounts
import uniffi.gemstone.GemNetworkAssetSections
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregates
import com.gemwallet.android.ui.models.navigation.requireChain
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import uniffi.gemstone.GemWalletHomeServiceInterface
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class NetworkAssetsViewModel @Inject constructor(
    assetStore: GemstoneAssetStore,
    getCurrentWalletId: GetCurrentWalletId,
    private val service: GemWalletHomeServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @ApplicationContext context: Context,
) : ViewModel() {

    private val chain: Chain = savedStateHandle.requireChain()

    val title: String = context.getString(R.string.assets_title)

    val rowStyle: GemAssetRowStyle = service.assetRowStyle()

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
        .map { it.counts().sections() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, assetGroups.value.counts().sections())

    init {
        viewModelScope.launch(ioDispatcher) {
            val loaded = assetGroups.first { it.isLoaded }
            runCatchingCancellable { service.updateBalances(loaded.assetIds()) }
                .onFailure { Log.e(TAG, "balances update failed for ${chain.string}", it) }
        }
    }

    private fun groups(active: List<AssetInfo>, hidden: List<AssetInfo>): NetworkAssetGroups {
        val tokens = active.tokens()
        val sections = assetConfig.assetSections(
            ids = tokens.map { it.asset.id.toIdentifier() },
            pinnedIds = tokens.filter { it.metadata.isPinned }.map { it.asset.id.toIdentifier() },
            showsPopular = false,
        )
        val byId = tokens.associateBy { it.asset.id.toIdentifier() }
        return NetworkAssetGroups(
            pinned = sections.pinned.mapNotNull(byId::get).toAssetInfoDataAggregates(rowStyle.title),
            unpinned = sections.assets.mapNotNull(byId::get).toAssetInfoDataAggregates(rowStyle.title),
            hidden = hidden.tokens().toAssetInfoDataAggregates(rowStyle.title),
            isLoaded = true,
        )
    }

    private fun List<AssetInfo>.tokens(): List<AssetInfo> = filter { it.asset.type != AssetType.NATIVE }

    fun hideAsset(assetId: AssetId) = setEnabled(assetId, false)

    fun addToWallet(assetId: AssetId) = setEnabled(assetId, true)

    fun togglePin(assetId: AssetId) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.setAssetPinned(assetId.toIdentifier(), assetGroups.value.pinned.none { it.id == assetId }) }
            .onFailure { Log.e(TAG, "pinning ${assetId.toIdentifier()} failed", it) }
    }

    private fun setEnabled(assetId: AssetId, enabled: Boolean) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.setAssetsEnabled(listOf(assetId.toIdentifier()), enabled) }
            .onFailure { Log.e(TAG, "setting ${assetId.toIdentifier()} enabled=$enabled failed", it) }
    }

    private companion object {
        const val TAG = "NetworkAssets"
    }
}

private data class NetworkAssetGroups(
    val pinned: List<AssetInfoDataAggregate> = emptyList(),
    val unpinned: List<AssetInfoDataAggregate> = emptyList(),
    val hidden: List<AssetInfoDataAggregate> = emptyList(),
    val isLoaded: Boolean = false,
) {
    fun counts(): GemNetworkAssetCounts = GemNetworkAssetCounts(
        pinned = pinned.size.toUInt(),
        unpinned = unpinned.size.toUInt(),
        hidden = hidden.size.toUInt(),
    )

    fun assetIds(): List<String> = (pinned + unpinned + hidden).map { it.id.toIdentifier() }
}
