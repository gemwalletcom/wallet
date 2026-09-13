package com.gemwallet.android.ui.theme

import uniffi.gemstone.walletAvatarEmojis

object AvatarEmoji {
    val all: List<String> by lazy { walletAvatarEmojis() }
}
