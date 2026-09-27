package com.gemwallet.android.features.wallet.viewmodels

interface InAppUpdateService {
    fun canRequestPackageInstalls(): Boolean

    suspend fun clearDownloadedUpdate()

    suspend fun download(url: String, version: String, onProgress: (Float?) -> Unit)

    fun installDownloadedUpdate(version: String)

    fun cancel()
}
