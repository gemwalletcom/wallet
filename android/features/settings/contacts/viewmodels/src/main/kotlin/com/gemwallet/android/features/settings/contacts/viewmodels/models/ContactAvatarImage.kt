package com.gemwallet.android.features.settings.contacts.viewmodels.models

import com.gemwallet.android.ui.components.list_item.ListItemImage
import uniffi.gemstone.GemContactAvatarImage

fun GemContactAvatarImage.listItemImage(emojiBackground: Int): ListItemImage = when (this) {
    is GemContactAvatarImage.Initials -> ListItemImage.Initials(text)
    GemContactAvatarImage.Placeholder -> ListItemImage.Initials("")
    is GemContactAvatarImage.Image -> ListItemImage.Stored(imageUrl, placeholder = initials)
    is GemContactAvatarImage.Emoji -> ListItemImage.Emoji(emoji, emojiBackground)
}
