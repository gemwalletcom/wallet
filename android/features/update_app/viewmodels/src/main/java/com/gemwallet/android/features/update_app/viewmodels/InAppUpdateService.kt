package com.gemwallet.android.features.update_app.viewmodels

interface InAppUpdateService {
    fun canRequestPackageInstalls(): Boolean

    suspend fun clearDownloadedUpdate()

    suspend fun download(url: String, version: String, onProgress: (Float?) -> Unit)

    fun installDownloadedUpdate(version: String)

    fun cancel()
}
