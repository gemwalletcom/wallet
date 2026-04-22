package com.gemwallet.android.features.nft.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.nft.coordinators.GetNftCollections
import com.gemwallet.android.application.nft.coordinators.SyncNftCollections
import com.gemwallet.android.cases.nft.NftError
import com.gemwallet.android.ui.models.NftItemUIModel
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.VerificationStatus
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

val collectionIdArg = "collectionId"
val unverifiedArg = "unverified"

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class NftListViewModels @Inject constructor(
    private val syncNftCollections: SyncNftCollections,
    private val getNftCollections: GetNftCollections,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val loadState = MutableStateFlow(false)

    val isLoading = loadState
        .flatMapLatest { shouldLoad ->
            flow {
                if (!shouldLoad) return@flow
                emit(true)
                runCatching { syncNftCollections() }
                emit(false)
                loadState.update { false }
            }
        }
        .catch { error.update { NftError.LoadError } }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val collectionId = savedStateHandle.getStateFlow<String?>(collectionIdArg, null)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val unverified = savedStateHandle.getStateFlow(unverifiedArg, false)
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private val nftData: StateFlow<List<NFTData>> = collectionId
        .onEach { loadState.emit(it == null) }
        .flatMapLatest { collectionId ->
            getNftCollections(collectionId).map { data -> data.filter { it.assets.isNotEmpty() } }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val collections = combine(nftData, collectionId, unverified) { data, collectionId, unverified ->
        val filtered = when {
            collectionId != null -> data
            unverified -> data.filter { it.collection.status != VerificationStatus.Verified }
            else -> data.filter { it.collection.status == VerificationStatus.Verified }
        }
        filtered.flatMap { nftData ->
            val isSingleAsset = nftData.assets.size == 1
            if (collectionId != null || isSingleAsset) {
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

    fun refresh() {
        viewModelScope.launch { loadState.emit(true) }
    }
}
