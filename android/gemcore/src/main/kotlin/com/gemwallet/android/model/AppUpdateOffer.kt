package com.gemwallet.android.model

data class AppUpdateOffer(
    val version: String,
    val isRequired: Boolean,
    val channel: AppUpdateChannel,
)

enum class AppUpdateChannel {
    Store,
    InAppApk,
}
