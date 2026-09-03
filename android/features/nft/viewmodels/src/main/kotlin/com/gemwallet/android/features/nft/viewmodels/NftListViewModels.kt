package com.gemwallet.android.features.nft.viewmodels

import uniffi.gemstone.GemNftServiceInterface
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.nft.cases.GetNftCollections
import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.models.NftItemUIModel
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class NftListViewModels @Inject constructor(
    private val nftService: GemNftServiceInterface,
    getNftCollections: GetNftCollections,
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    val mode: NftListMode = savedStateHandle.nftListMode()

    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing

    private val session = getSession()

    val walletId: StateFlow<WalletId?> = session
        .map { it?.wallet?.id }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private var lastSyncedWalletId: WalletId? = null

    private val nftData: StateFlow<List<NFTData>> = getNftCollections(mode.collectionId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val collections = nftData
        .map { data ->
            val filtered = when (mode) {
                is NftListMode.Collection -> data.filter { it.assets.isNotEmpty() }
                NftListMode.Unverified -> nftService.unverifiedCollections(data.map { it.toJson() }).map { it.decodeJson<NFTData>() }
                NftListMode.Collections -> nftService.verifiedCollections(data.map { it.toJson() }).map { it.decodeJson<NFTData>() }
            }
            nftService.sortedCollections(filtered.map { it.toJson() }).map { it.decodeJson<NFTData>() }.flatMap { nftData ->
                val isSingleAsset = nftData.assets.size == 1
                if (mode is NftListMode.Collection || isSingleAsset) {
                    nftData.assets.map { NftItemUIModel(nftData.collection, it) }
                } else {
                    listOf(NftItemUIModel(nftData.collection, null, nftData.assets.size))
                }
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val unverifiedCount = nftData
        .map { data -> nftService.unverifiedCollections(data.map { it.toJson() }).size }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    fun syncIfNeeded() {
        if (mode is NftListMode.Collection) return
        val current = walletId.value ?: return
        if (current == lastSyncedWalletId) return
        lastSyncedWalletId = current
        viewModelScope.launch(Dispatchers.IO) {
            sync()
        }
    }

    fun refresh() {
        viewModelScope.launch(Dispatchers.IO) {
            _isRefreshing.update { true }
            try {
                sync()
            } finally {
                _isRefreshing.update { false }
            }
        }
    }

    private suspend fun sync() {
        runCatchingCancellable { nftService.sync() }
            .onFailure { Log.e(TAG, "nft collections sync failed", it) }
    }

    private companion object {
        const val TAG = "NftList"
    }
}
