package com.gemwallet.android.features.settings.contacts.viewmodels.models

sealed interface ContactAvatarState {
    data object Empty : ContactAvatarState
    data class Image(val imageUrl: String) : ContactAvatarState
    data class Emoji(val emoji: String, val backgroundColor: Int) : ContactAvatarState

    companion object {
        fun from(imageUrl: String?): ContactAvatarState = imageUrl?.let { Image(it) } ?: Empty
    }
}
