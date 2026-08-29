package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.cases.GetWalletDetails
import com.gemwallet.android.application.nft.cases.GetListNft
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.theme.AvatarEmoji
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject
import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable

@HiltViewModel
class WalletImageViewModel @Inject constructor(
    getWalletDetails: GetWalletDetails,
    getListNftCase: GetListNft,
    private val avatarService: WalletAvatarService,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val walletId = savedStateHandle.requireWalletId()

    val wallet = getWalletDetails.getWallet(walletId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val emojis: List<String> = AvatarEmoji.all

    val nftImages: StateFlow<List<NftItemUIModel>> = getListNftCase.getListNft(walletId)
        .map { data ->
            data.flatMap { nftData -> nftData.assets.map { NftItemUIModel(nftData.collection, it) } }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun setEmoji(emoji: String, backgroundColor: Int) = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { avatarService.setEmoji(walletId, emoji, backgroundColor) }
            .onFailure { Log.e(TAG, "wallet emoji update failed", it) }
    }

    fun setNftImage(url: String) = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { avatarService.setNftImage(walletId, url) }
            .onFailure { Log.e(TAG, "wallet image update failed", it) }
    }

    fun resetToDefault() = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { avatarService.reset(walletId) }
            .onFailure { Log.e(TAG, "wallet image reset failed", it) }
    }

    private companion object {
        const val TAG = "WalletImage"
    }

}
