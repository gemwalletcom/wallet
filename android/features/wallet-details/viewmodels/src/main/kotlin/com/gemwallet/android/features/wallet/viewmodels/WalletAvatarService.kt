package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import coil3.imageLoader
import coil3.request.ErrorResult
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import coil3.toBitmap
import com.gemwallet.android.application.wallet.coordinators.SetWalletAvatar
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class WalletAvatarService @Inject constructor(
    @ApplicationContext private val context: Context,
    private val emojiRenderer: EmojiAvatarRenderer,
    private val localStore: LocalStore,
    private val setWalletAvatar: SetWalletAvatar,
) {

    suspend fun setEmoji(walletId: WalletId, currentImageUrl: String?, emoji: String, backgroundColor: Int) =
        saveAvatar(walletId, currentImageUrl) { emojiRenderer.render(emoji, backgroundColor) }

    suspend fun setNftImage(walletId: WalletId, currentImageUrl: String?, url: String) =
        saveAvatar(walletId, currentImageUrl) { loadImage(url) }

    suspend fun reset(walletId: WalletId, currentImageUrl: String?) {
        withContext(Dispatchers.IO) { localStore.remove(currentImageUrl) }
        setWalletAvatar.setWalletAvatar(walletId, null)
    }

    private suspend fun saveAvatar(walletId: WalletId, currentImageUrl: String?, produce: suspend () -> ByteArray) {
        val fileName = withContext(Dispatchers.IO) {
            val data = produce()
            localStore.remove(currentImageUrl)
            localStore.save(data, IMAGE_EXTENSION)
        }
        setWalletAvatar.setWalletAvatar(walletId, fileName)
    }

    private suspend fun loadImage(url: String): ByteArray {
        val request = ImageRequest.Builder(context).data(url).build()
        return when (val result = context.imageLoader.execute(request)) {
            is SuccessResult -> result.image.toBitmap().toPng()
            is ErrorResult -> throw result.throwable
        }
    }

    private companion object {
        const val IMAGE_EXTENSION = "png"
    }
}
