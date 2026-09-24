package com.gemwallet.android.features.nft.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.nft.viewmodels.models.NftDetailsUIModel
import com.gemwallet.android.features.nft.viewmodels.models.ReportReasonUIModel
import com.gemwallet.android.features.nft.viewmodels.models.uiModel
import com.gemwallet.android.ui.components.image.canSaveImageToGallery
import com.gemwallet.android.ui.components.image.saveImageToGallery
import com.gemwallet.android.ui.models.navigation.requireNftAssetId
import com.wallet.core.primitives.ReportNft
import com.wallet.core.primitives.ReportReason
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemCollectibleServiceInterface
import javax.inject.Inject

@HiltViewModel
class NftDetailsViewModel @Inject constructor(
    getNftAssetDetails: GetNftAssetDetails,
    private val service: GemCollectibleServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val nftAssetId = savedStateHandle.requireNftAssetId()

    private val details = getNftAssetDetails(nftAssetId, canSaveImage = canSaveImageToGallery)
        .catch { Log.e(TAG, "Collectible details unavailable", it) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val nftAsset: StateFlow<NftDetailsUIModel?> = details
        .map { it?.uiModel(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val reportReasons: List<ReportReasonUIModel> = ReportReason.entries.map { it.uiModel(context) }

    suspend fun refresh(): Boolean = withContext(ioDispatcher) {
        runCatchingCancellable { service.refreshAsset(nftAssetId.toIdentifier()) }.isSuccess
    }

    suspend fun setAsAvatar(): Boolean = withContext(ioDispatcher) {
        val url = details.value?.asset?.images?.preview?.url ?: return@withContext false
        runCatchingCancellable { service.setWalletAvatar(url) }.isSuccess
    }

    suspend fun saveImage(): Boolean = withContext(ioDispatcher) {
        val asset = details.value?.asset ?: return@withContext false
        runCatchingCancellable { context.saveImageToGallery(url = asset.images.preview.url, name = asset.name) }.isSuccess
    }

    suspend fun report(reason: ReportReason): Boolean = withContext(ioDispatcher) {
        runCatchingCancellable { service.report(nftAssetId.toIdentifier(), reason.toGem()) }.isSuccess
    }
}

private const val TAG = "NftDetailsViewModel"
