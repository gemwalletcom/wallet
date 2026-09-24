package com.gemwallet.android.model

data class AppUpdateOffer(val version: String, val canSkip: Boolean, val channel: AppUpdateChannel)

enum class AppUpdateChannel {
    Store,
    InAppApk,
}
