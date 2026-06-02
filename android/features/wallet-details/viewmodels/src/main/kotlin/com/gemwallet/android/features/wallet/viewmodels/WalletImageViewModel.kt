package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.coordinators.GetWalletDetails
import com.gemwallet.android.cases.nft.GetListNftCase
import com.gemwallet.android.ui.models.NftItemUIModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class WalletImageViewModel @Inject constructor(
    getWalletDetails: GetWalletDetails,
    getListNftCase: GetListNftCase,
    private val avatarService: WalletAvatarService,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val walletId = savedStateHandle.requireWalletId()

    val wallet = getWalletDetails.getWallet(walletId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val emojis: List<String> = WalletAvatarEmoji.all

    val nftImages: StateFlow<List<NftItemUIModel>> = getListNftCase.getListNft(walletId)
        .map { data ->
            data.flatMap { nftData -> nftData.assets.map { NftItemUIModel(nftData.collection, it) } }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun setEmoji(emoji: String, backgroundColor: Int) = viewModelScope.launch(Dispatchers.IO) {
        avatarService.setEmoji(walletId, wallet.value?.imageUrl, emoji, backgroundColor)
    }

    fun setNftImage(url: String) = viewModelScope.launch(Dispatchers.IO) {
        runCatching { avatarService.setNftImage(walletId, wallet.value?.imageUrl, url) }
    }

    fun resetToDefault() = viewModelScope.launch(Dispatchers.IO) {
        avatarService.reset(walletId, wallet.value?.imageUrl)
    }
}
