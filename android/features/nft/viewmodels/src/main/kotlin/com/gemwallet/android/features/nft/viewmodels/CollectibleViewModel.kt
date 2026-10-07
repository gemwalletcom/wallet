package com.gemwallet.android.features.nft.viewmodels

import android.content.Context
import androidx.annotation.StringRes
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.nft.NFTAssetDetails
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.nft.viewmodels.models.CollectibleUIModel
import com.gemwallet.android.features.nft.viewmodels.models.ReportReasonUIModel
import com.gemwallet.android.features.nft.viewmodels.models.uiModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.canSaveImageToGallery
import com.gemwallet.android.ui.components.image.saveImageToGallery
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.ToastEmitter
import com.gemwallet.android.ui.models.ToastEmitterImpl
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.dataOrNull
import com.gemwallet.android.ui.models.flatMap
import com.gemwallet.android.ui.models.navigation.requireNftAssetId
import com.wallet.core.primitives.ReportReason
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemCollectibleServiceInterface
import javax.inject.Inject

@HiltViewModel
class CollectibleViewModel @Inject constructor(
    getNftAssetDetails: GetNftAssetDetails,
    getSession: GetSession,
    private val service: GemCollectibleServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel(),
    ToastEmitter by ToastEmitterImpl() {

    private val nftAssetId = savedStateHandle.requireNftAssetId()

    private val assetDetails: StateFlow<StateViewType<NFTAssetDetails>> = getNftAssetDetails(nftAssetId)
        .map<NFTAssetDetails, StateViewType<NFTAssetDetails>> { StateViewType.Data(it) }
        .catch { emit(StateViewType.Error(it.errorText().text(context))) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, StateViewType.Loading)

    val state: StateFlow<StateViewType<CollectibleUIModel>> = combine(assetDetails, getSession().filterNotNull()) { asset, session ->
        asset.flatMap { StateViewType.Data(CollectibleUIModel(it.assetData, service.details(session.wallet.type.toGem(), it.assetData.toGem(), it.isOwned, canSaveImageToGallery))) }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, StateViewType.Loading)

    val reportReasons: List<ReportReasonUIModel> = ReportReason.entries.map { it.uiModel(context) }

    fun refresh() = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.refreshAsset(nftAssetId.toIdentifier()) }
            .toast(R.string.common_refresh)
    }

    fun setAsAvatar() = viewModelScope.launch(ioDispatcher) {
        val url = state.value.dataOrNull?.assetData?.asset?.images?.preview?.url ?: return@launch
        runCatchingCancellable { service.setWalletAvatar(url) }
            .toast(R.string.nft_set_as_avatar)
    }

    fun saveImage() = viewModelScope.launch(ioDispatcher) {
        val asset = state.value.dataOrNull?.assetData?.asset ?: return@launch
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
