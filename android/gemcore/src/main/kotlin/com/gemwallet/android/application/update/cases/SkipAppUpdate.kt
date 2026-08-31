package com.gemwallet.android.application.update.cases

interface SkipAppUpdate {
    suspend fun skipAppUpdate(version: String)
}
