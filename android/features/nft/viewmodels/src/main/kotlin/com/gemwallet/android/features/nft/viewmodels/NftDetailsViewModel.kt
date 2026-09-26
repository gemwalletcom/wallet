package com.gemwallet.android.features.nft.viewmodels

import android.content.Context
import android.util.Log
import androidx.annotation.StringRes
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.nft.viewmodels.models.NftDetailsUIModel
import com.gemwallet.android.features.nft.viewmodels.models.ReportReasonUIModel
import com.gemwallet.android.features.nft.viewmodels.models.uiModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.canSaveImageToGallery
import com.gemwallet.android.ui.components.image.saveImageToGallery
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ToastEmitter
import com.gemwallet.android.ui.models.ToastEmitterImpl
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.navigation.requireNftAssetId
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
import kotlinx.coroutines.launch
import uniffi.gemstone.GemCollectibleServiceInterface
import javax.inject.Inject

@HiltViewModel
class NftDetailsViewModel @Inject constructor(
    getNftAssetDetails: GetNftAssetDetails,
    private val service: GemCollectibleServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel(),
    ToastEmitter by ToastEmitterImpl() {

    private val nftAssetId = savedStateHandle.requireNftAssetId()

    private val details = getNftAssetDetails(nftAssetId, canSaveImage = canSaveImageToGallery)
        .catch { Log.e(TAG, "Collectible details unavailable", it) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val nftAsset: StateFlow<NftDetailsUIModel?> = details
        .map { it?.uiModel(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val reportReasons: List<ReportReasonUIModel> = ReportReason.entries.map { it.uiModel(context) }

    fun refresh() = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.refreshAsset(nftAssetId.toIdentifier()) }
            .toast(R.string.common_refresh)
    }

    fun setAsAvatar() = viewModelScope.launch(ioDispatcher) {
        val url = details.value?.asset?.images?.preview?.url ?: return@launch
        runCatchingCancellable { service.setWalletAvatar(url) }
            .toast(R.string.nft_set_as_avatar)
    }

    fun saveImage() = viewModelScope.launch(ioDispatcher) {
        val asset = details.value?.asset ?: return@launch
        runCatchingCancellable { context.saveImageToGallery(url = asset.images.preview.url, name = asset.name) }
            .toast(R.string.nft_save_to_photos)
    }

    fun report(reason: ReportReason) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.report(nftAssetId.toIdentifier(), reason.toGem()) }
            .toast(R.string.transaction_status_confirmed)
    }

    private fun Result<*>.toast(@StringRes success: Int) {
        onSuccess { emitToast(ToastMessage(context.getString(success), R.drawable.ic_check_circle)) }
        onFailure { emitToast(ToastMessage(it.errorText().text(context), R.drawable.ic_error)) }
    }
}

private const val TAG = "NftDetailsViewModel"
