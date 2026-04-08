package com.gemwallet.android.features.nft.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.nft.coordinators.GetNftAssetDetails
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

val nftAssetIdArg = "assetId"

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class NftDetailsViewModel @Inject constructor(
    private val getNftAssetDetails: GetNftAssetDetails,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val assetId = savedStateHandle.getStateFlow<String?>(nftAssetIdArg, null)

    val nftAsset = assetId
        .filterNotNull()
        .flatMapLatest { getNftAssetDetails(it) }
        .catch { }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
}
