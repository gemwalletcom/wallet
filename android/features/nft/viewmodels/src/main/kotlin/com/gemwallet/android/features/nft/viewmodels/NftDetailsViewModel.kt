package com.gemwallet.android.features.nft.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.nft.coordinators.GetNftAssetDetails
import com.gemwallet.android.application.nft.coordinators.RefreshNftAsset
import com.gemwallet.android.ext.toAssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext
import javax.inject.Inject

val nftAssetIdArg = "assetId"

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class NftDetailsViewModel @Inject constructor(
    private val getNftAssetDetails: GetNftAssetDetails,
    private val refreshNftAsset: RefreshNftAsset,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val assetIdState = savedStateHandle.getStateFlow<String?>(nftAssetIdArg, null)

    val nftAsset = assetIdState
        .mapNotNull { it?.toAssetId() }
        .flatMapLatest { getNftAssetDetails(it) }
        .catch { }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    suspend fun refresh(): Boolean {
        val currentAssetId = assetIdState.value?.toAssetId() ?: return false
        return try {
            withContext(Dispatchers.IO) {
                refreshNftAsset(currentAssetId)
            }
            true
        } catch (error: CancellationException) {
            throw error
        } catch (_: Throwable) {
            false
        }
    }
}
