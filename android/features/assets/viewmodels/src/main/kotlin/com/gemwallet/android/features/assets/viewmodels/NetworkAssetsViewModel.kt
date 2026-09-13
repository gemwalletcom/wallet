package com.gemwallet.android.features.assets.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import android.util.Log
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import uniffi.gemstone.GemAssetRow
import uniffi.gemstone.GemNetworkAssetCounts
import uniffi.gemstone.GemNetworkAssetSections
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregates
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
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
    @ApplicationContext context: Context,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val chain: Chain = Chain.entries.first { it.string == savedStateHandle.get<String>(RouteArgument.Chain.key) }

    val title: String = context.getString(R.string.assets_title)

    val row: GemAssetRow = service.assetRow()

    private val activeAssets = getCurrentWalletId()
        .flatMapLatest { walletId -> assetStore.observeAssetsInfoByChain(walletId.id, chain) }
        .map { assets -> assets.filter { it.asset.type != AssetType.NATIVE } }
        .flowOn(Dispatchers.IO)

    val pinned: StateFlow<List<AssetInfoDataAggregate>> = activeAssets
        .map { assets -> assets.filter { it.metadata.isPinned }.toAssetInfoDataAggregates(row.title) }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val unpinned: StateFlow<List<AssetInfoDataAggregate>> = activeAssets
        .map { assets -> assets.filter { !it.metadata.isPinned }.toAssetInfoDataAggregates(row.title) }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val hiddenAssets = getCurrentWalletId()
        .flatMapLatest { walletId -> assetStore.observeHiddenAssetsInfoByChain(walletId.id, chain) }
        .map { assets -> assets.filter { it.asset.type != AssetType.NATIVE } }
        .flowOn(Dispatchers.IO)

    val hidden: StateFlow<List<AssetInfoDataAggregate>> = hiddenAssets
        .map { assets -> assets.toAssetInfoDataAggregates(row.title) }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val sections: StateFlow<GemNetworkAssetSections> = combine(pinned, unpinned, hidden) { pinned, unpinned, hidden ->
        GemNetworkAssetCounts(
            pinned = pinned.size.toUInt(),
            unpinned = unpinned.size.toUInt(),
            hidden = hidden.size.toUInt(),
        ).sections()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, GemNetworkAssetCounts(0u, 0u, 0u).sections())

    init {
        viewModelScope.launch(Dispatchers.IO) {
            val assetIds = (activeAssets.first() + hiddenAssets.first()).map { it.asset.id.toIdentifier() }
            runCatchingCancellable { service.updateBalances(assetIds) }
                .onFailure { Log.e(TAG, "balances update failed for ${chain.string}", it) }
        }
    }

    fun hideAsset(assetId: AssetId) = setEnabled(assetId, false)

    fun addToWallet(assetId: AssetId) = setEnabled(assetId, true)

    fun togglePin(assetId: AssetId) = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { service.setAssetPinned(assetId.toIdentifier(), pinned.value.none { it.id == assetId }) }
            .onFailure { Log.e(TAG, "pinning ${assetId.toIdentifier()} failed", it) }
    }

    private fun setEnabled(assetId: AssetId, enabled: Boolean) = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { service.setAssetsEnabled(listOf(assetId.toIdentifier()), enabled) }
            .onFailure { Log.e(TAG, "setting ${assetId.toIdentifier()} enabled=$enabled failed", it) }
    }

    private companion object {
        const val TAG = "NetworkAssets"
    }
}
