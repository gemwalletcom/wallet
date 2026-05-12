package com.gemwallet.android.features.nft.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.nft.coordinators.GetNftCollections
import com.gemwallet.android.application.nft.coordinators.SyncNftCollections
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.cases.nft.NftError
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.VerificationStatus
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class NftListViewModels @Inject constructor(
    private val syncNftCollections: SyncNftCollections,
    private val getNftCollections: GetNftCollections,
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing

    val nftCollectionId = savedStateHandle.getStateFlow<String?>(RouteArgument.NftCollectionId.key, null)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val unverified = savedStateHandle.getStateFlow(RouteArgument.Unverified.key, false)
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private val session = getSession()

    val walletId: StateFlow<WalletId?> = session
        .map { it?.wallet?.id }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private var lastSyncedWalletId: WalletId? = null

    private val nftData: StateFlow<List<NFTData>> = nftCollectionId
        .flatMapLatest { collectionId ->
            getNftCollections(collectionId).map { data -> data.filter { it.assets.isNotEmpty() } }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val collections = combine(nftData, nftCollectionId, unverified) { data, nftCollectionId, unverified ->
        val filtered = when {
            nftCollectionId != null -> data
            unverified -> data.filter { it.collection.status != VerificationStatus.Verified }
            else -> data.filter { it.collection.status == VerificationStatus.Verified }
        }
        filtered.flatMap { nftData ->
            val isSingleAsset = nftData.assets.size == 1
            if (nftCollectionId != null || isSingleAsset) {
                nftData.assets.map { NftItemUIModel(nftData.collection, it) }
            } else {
                listOf(NftItemUIModel(nftData.collection, null, nftData.assets.size))
            }
        }
        .sortedBy { it.name }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val unverifiedCount = nftData
        .map { data -> data.count { it.collection.status != VerificationStatus.Verified } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    val error = MutableStateFlow<NftError?>(null)

    fun syncIfNeeded() {
        if (nftCollectionId.value != null) return
        val current = walletId.value ?: return
        if (current == lastSyncedWalletId) return
        lastSyncedWalletId = current
        viewModelScope.launch(Dispatchers.IO) {
            val walletId = session.firstOrNull()?.wallet?.id ?: return@launch
            syncNftCollections.syncNftCollections(walletId)
        }
    }

    fun refresh() {
        viewModelScope.launch(Dispatchers.IO) {
            val walletId = session.firstOrNull()?.wallet?.id ?: return@launch
            _isRefreshing.update { true }
            try {
                syncNftCollections.syncNftCollections(walletId)
            } finally {
                _isRefreshing.update { false }
            }
        }
    }
}
