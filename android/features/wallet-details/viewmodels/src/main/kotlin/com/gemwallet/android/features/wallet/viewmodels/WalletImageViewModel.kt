package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.cases.GetWalletDetails
import com.gemwallet.android.application.nft.cases.GetListNft
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.theme.AvatarEmoji
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject
import com.gemwallet.android.ext.runCatchingCancellable
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ui.models.toUIModels
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.NFTAssetData
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemNftItem

@HiltViewModel
class WalletImageViewModel @Inject constructor(
    getWalletDetails: GetWalletDetails,
    getListNftCase: GetListNft,
    private val avatarService: WalletAvatarService,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val walletId = savedStateHandle.requireWalletId()

    val wallet = getWalletDetails.getWallet(walletId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val emojis: List<String> = AvatarEmoji.all

    val nftImages: StateFlow<List<NftItemUIModel>> = getListNftCase.getListNft(walletId)
        .map { data ->
            data.flatMap { nftData -> nftData.assets.map { asset -> GemNftItem.Asset(NFTAssetData(nftData.collection, asset).toGem()) } }.toUIModels()
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val errorState = MutableStateFlow<GemErrorText?>(null)
    val error: StateFlow<GemErrorText?> = errorState.asStateFlow()

    fun setEmoji(emoji: String, backgroundColor: Int) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { avatarService.setEmoji(walletId, emoji, backgroundColor) }
            .onFailure { errorState.value = it.errorText() }
    }

    fun setNftImage(url: String) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { avatarService.setNftImage(walletId, url) }
            .onFailure { errorState.value = it.errorText() }
    }

    fun resetToDefault() = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { avatarService.reset(walletId) }
            .onFailure { errorState.value = it.errorText() }
    }

    fun clearError() = errorState.update { null }


}
