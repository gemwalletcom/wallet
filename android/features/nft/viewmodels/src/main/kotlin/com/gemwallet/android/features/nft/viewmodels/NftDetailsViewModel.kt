package com.gemwallet.android.features.nft.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.nft.coordinators.GetNftAssetDetails
import com.gemwallet.android.application.nft.coordinators.RefreshNftAsset
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext
import javax.inject.Inject

@HiltViewModel
class NftDetailsViewModel @Inject constructor(
    private val getNftAssetDetails: GetNftAssetDetails,
    private val refreshNftAsset: RefreshNftAsset,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val nftAssetId = savedStateHandle.requireAssetId(RouteArgument.NftAssetId)

    val nftAsset = getNftAssetDetails(nftAssetId)
        .catch { }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    suspend fun refresh(): Boolean {
        return try {
            withContext(Dispatchers.IO) {
                refreshNftAsset(nftAssetId)
            }
            true
        } catch (error: CancellationException) {
            throw error
        } catch (_: Throwable) {
            false
        }
    }
}
