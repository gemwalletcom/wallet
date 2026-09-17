package com.gemwallet.android.features.settings.contacts.viewmodels.models

import com.gemwallet.android.ui.components.list_item.ListItemImage

sealed interface ContactAvatarState {
    data object Empty : ContactAvatarState
    data class Image(val imageUrl: String) : ContactAvatarState
    data class Emoji(val emoji: String, val backgroundColor: Int) : ContactAvatarState

    companion object {
        fun from(imageUrl: String?): ContactAvatarState = imageUrl?.let { Image(it) } ?: Empty
    }
}

fun ContactAvatarState.image(initials: String): ListItemImage = when (this) {
    ContactAvatarState.Empty -> ListItemImage.Initials(initials)
    is ContactAvatarState.Image -> ListItemImage.Stored(imageUrl, placeholder = initials)
    is ContactAvatarState.Emoji -> ListItemImage.Emoji(emoji, backgroundColor)
}
