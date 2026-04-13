package com.gemwallet.android.application.fiat.coordinators

interface SyncFiatAssets {
    suspend operator fun invoke()
}
