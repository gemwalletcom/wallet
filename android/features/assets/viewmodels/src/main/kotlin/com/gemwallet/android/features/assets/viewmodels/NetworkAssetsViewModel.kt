package com.gemwallet.android.features.assets.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.asset_select.coordinators.GetChainAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.assets.coordinators.HideAsset
import com.gemwallet.android.application.assets.coordinators.ToggleAssetPin
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.aggregates.AssetRowNaming
import com.gemwallet.android.domains.asset.aggregates.toAssetInfoDataAggregate
import com.gemwallet.android.ui.R
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
import javax.inject.Inject

@HiltViewModel
class NetworkAssetsViewModel @Inject constructor(
    getChainAssets: GetChainAssets,
    private val getSession: GetSession,
    private val hideAsset: HideAsset,
    private val toggleAssetPin: ToggleAssetPin,
    private val switchAssetVisibility: SwitchAssetVisibility,
    @ApplicationContext context: Context,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val chain: Chain = Chain.entries.first { it.string == savedStateHandle.get<String>(RouteArgument.Chain.key) }

    val title: String = context.getString(R.string.assets_title)

    private val activeAssets = getChainAssets(chain)
        .map { assets -> assets.filter { it.asset.type != AssetType.NATIVE } }
        .flowOn(Dispatchers.IO)

    val pinned: StateFlow<List<AssetInfoDataAggregate>> = activeAssets
        .map { assets -> assets.filter { it.metadata?.isPinned == true }.map { it.toAssetInfoDataAggregate(AssetRowNaming.CanonicalNative) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val unpinned: StateFlow<List<AssetInfoDataAggregate>> = activeAssets
        .map { assets -> assets.filter { it.metadata?.isPinned != true }.map { it.toAssetInfoDataAggregate(AssetRowNaming.CanonicalNative) } }
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

    fun hideAsset(assetId: AssetId) = viewModelScope.launch {
        hideAsset.invoke(assetId)
    }

    fun togglePin(assetId: AssetId) = viewModelScope.launch {
        toggleAssetPin(assetId)
    }

    fun addToWallet(assetId: AssetId) = viewModelScope.launch {
        val walletId = getSession().value?.wallet?.id ?: return@launch
        switchAssetVisibility(walletId, assetId, true)
    }
}
