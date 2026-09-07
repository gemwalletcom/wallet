package com.gemwallet.android.features.nft.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.models.navigation.requireNftAssetId
import com.wallet.core.primitives.ReportReason
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemCollectibleServiceInterface
import javax.inject.Inject
import com.wallet.core.primitives.ReportNft
import com.gemwallet.android.ext.toGem

@HiltViewModel
class NftDetailsViewModel @Inject constructor(
    getNftAssetDetails: GetNftAssetDetails,
    private val service: GemCollectibleServiceInterface,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val nftAssetId = savedStateHandle.requireNftAssetId()

    val nftAsset = getNftAssetDetails(nftAssetId)
        .catch { }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    suspend fun refresh(): Boolean = withContext(Dispatchers.IO) {
        runCatchingCancellable { service.refreshAsset(nftAssetId.toIdentifier()) }.isSuccess
    }

    suspend fun report(reason: ReportReason): Boolean = withContext(Dispatchers.IO) {
        val asset = nftAsset.value ?: return@withContext false
        val report = ReportNft(
            collectionId = asset.collection.id.toIdentifier(),
            assetId = nftAssetId.toIdentifier(),
            reason = reason.string,
        )
        runCatchingCancellable { service.report(report.toGem()) }.isSuccess
    }
}
