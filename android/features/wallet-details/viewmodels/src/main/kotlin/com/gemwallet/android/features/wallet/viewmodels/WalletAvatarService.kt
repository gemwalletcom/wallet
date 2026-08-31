package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import com.gemwallet.android.ui.components.image.EmojiAvatarRenderer
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.qualifiers.ApplicationContext
import uniffi.gemstone.GemAvatarService
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class WalletAvatarService @Inject constructor(
    @param:ApplicationContext private val context: Context,
    private val avatarService: GemAvatarService,
) {
    suspend fun setEmoji(walletId: WalletId, emoji: String, backgroundColor: Int) =
        avatarService.setImage(walletId.id, EmojiAvatarRenderer.render(context, emoji, backgroundColor))

    suspend fun setNftImage(walletId: WalletId, url: String) = avatarService.setImageUrl(walletId.id, url)

    suspend fun reset(walletId: WalletId) = avatarService.removeImage(walletId.id)
}
