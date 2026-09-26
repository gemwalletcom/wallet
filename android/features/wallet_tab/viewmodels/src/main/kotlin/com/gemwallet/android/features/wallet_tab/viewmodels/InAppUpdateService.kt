package com.gemwallet.android.features.wallet_tab.viewmodels

interface InAppUpdateService {
    fun canRequestPackageInstalls(): Boolean

    suspend fun clearDownloadedUpdate()

    suspend fun download(url: String, version: String, onProgress: (Float?) -> Unit)

    fun installDownloadedUpdate(version: String)

    fun cancel()
}
