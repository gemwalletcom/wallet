package com.gemwallet.android.features.wallets.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.data.services.store.queries.NFTQuery
import com.gemwallet.android.data.services.store.queries.WalletQuery
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.components.image.EmojiAvatarRenderer
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.theme.AvatarEmoji
import com.wallet.core.primitives.NFTAssetData
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemNftEntry
import uniffi.gemstone.GemWalletDetails
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class WalletImageViewModel @Inject constructor(
    walletQuery: WalletQuery,
    nftQuery: NFTQuery,
    private val walletService: GemWalletServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val walletId = savedStateHandle.requireWalletId()

    val details: StateFlow<GemWalletDetails?> = walletQuery(walletId)
        .mapLatest { wallet -> wallet?.let { walletService.walletDetails(it.toGem()) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val emojis: List<String> = AvatarEmoji.all

    val nftImages: StateFlow<List<GemNftEntry>> = nftQuery(walletId.id)
        .map { data -> walletService.avatarItems(data.map { it.toGem() }) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    fun setEmoji(emoji: String, backgroundColor: Int) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { walletService.setAvatarImage(walletId.id, EmojiAvatarRenderer.render(context, emoji, backgroundColor)) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun setNftImage(url: String) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { walletService.setAvatarImageUrl(walletId.id, url) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun resetToDefault() = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { walletService.removeAvatarImage(walletId.id) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun clearError() = errorState.update { null }
}
