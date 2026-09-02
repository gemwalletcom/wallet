package com.gemwallet.android.features.assets.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import android.util.Log
import com.gemwallet.android.application.asset_select.cases.GetChainAssets
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.AssetRowNaming
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemWalletHomeServiceInterface
import javax.inject.Inject

@HiltViewModel
class NetworkAssetsViewModel @Inject constructor(
    getChainAssets: GetChainAssets,
    private val service: GemWalletHomeServiceInterface,
    @ApplicationContext context: Context,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val chain: Chain = Chain.entries.first { it.string == savedStateHandle.get<String>(RouteArgument.Chain.key) }

    val title: String = context.getString(R.string.assets_title)

    private val activeAssets = getChainAssets(chain)
        .map { assets -> assets.filter { it.asset.type != AssetType.NATIVE } }
        .flowOn(Dispatchers.IO)

    val pinned: StateFlow<List<AssetInfoDataAggregate>> = activeAssets
        .map { assets -> assets.filter { it.metadata.isPinned }.map { it.toAssetInfoDataAggregate(AssetRowNaming.CanonicalNative) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val unpinned: StateFlow<List<AssetInfoDataAggregate>> = activeAssets
        .map { assets -> assets.filter { !it.metadata.isPinned }.map { it.toAssetInfoDataAggregate(AssetRowNaming.CanonicalNative) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val hidden: StateFlow<List<AssetInfoDataAggregate>> = getChainAssets.hidden(chain)
        .map { assets -> assets.filter { it.asset.type != AssetType.NATIVE }.map { it.toAssetInfoDataAggregate(AssetRowNaming.CanonicalNative) } }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val isEmpty: StateFlow<Boolean> = combine(pinned, unpinned, hidden) { pinned, unpinned, hidden ->
        pinned.isEmpty() && unpinned.isEmpty() && hidden.isEmpty()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    init {
        viewModelScope.launch(Dispatchers.IO) {
            getChainAssets.updateBalances(chain)
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
