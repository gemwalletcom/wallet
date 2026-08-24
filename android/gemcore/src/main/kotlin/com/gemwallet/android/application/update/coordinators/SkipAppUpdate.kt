package com.gemwallet.android.application.update.coordinators

interface SkipAppUpdate {
    suspend fun skipAppUpdate(version: String)
}
