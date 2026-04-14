package com.gemwallet.android.features.update_app.viewmodels

import android.content.Context
import android.content.Intent
import android.content.Intent.FLAG_ACTIVITY_NEW_TASK
import android.os.Environment
import androidx.core.content.FileProvider
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.cases.config.GetLatestVersion
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.ext.universalApkDownloadUrl
import com.gemwallet.android.ext.VersionCheck
import com.gemwallet.android.model.BuildInfo
import com.wallet.core.primitives.PlatformStore
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import okhttp3.OkHttpClient
import okhttp3.Request
import okio.buffer
import okio.sink
import java.io.File
import java.util.concurrent.TimeUnit
import javax.inject.Inject

@HiltViewModel
class InAppUpdateViewModels @Inject constructor(
    @param:ApplicationContext private val context: Context,
    private val buildInfo: BuildInfo,
    private val getLatestVersion: GetLatestVersion,
    private val userConfig: UserConfig,
) : ViewModel() {

    private val appFileProvider = "${context.packageName}.provider"
    private val intentDataType = "application/vnd.android.package-archive"

    val updateAvailable = userConfig.getAppVersionSkip().combine(getLatestVersion.getLatestVersion()) { skip, version ->
        if (version != skip
            && VersionCheck.isVersionHigher(new = version, current = buildInfo.versionName)
            && isApkStore(buildInfo.platformStore)
        ) {
            version
        } else {
            null
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val downloadState = MutableStateFlow<DownloadState>(DownloadState.Idle)

    init {
        viewModelScope.launch(Dispatchers.IO) {
            runCatching { getApkFile().delete() }
        }
    }

    fun update(): Boolean {
        if (!context.packageManager.canRequestPackageInstalls()) {
            return false
        }

        viewModelScope.launch(Dispatchers.IO) {
            downloadState.update { DownloadState.Preparing }
            try {
                download()
                if (downloadState.value == DownloadState.Success) {
                    installApk()
                }
            } catch (_: Throwable) {
                downloadState.update { DownloadState.Error }
                runCatching { getApkFile().delete() }
            }
        }
        return true
    }

    fun skip() {
        viewModelScope.launch {
            userConfig.setAppVersionSkip(updateAvailable.value ?: return@launch)
        }
    }

    private fun download() {
        val version = updateAvailable.value ?: throw IllegalArgumentException()
        val url = universalApkDownloadUrl(version)
        val destinationFile = getApkFile()

        val client = OkHttpClient.Builder()
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(60, TimeUnit.SECONDS)
            .build()

        val request = Request.Builder().url(url).build()
        val response = client.newCall(request).execute()

        if (!response.isSuccessful) {
            response.close()
            throw IllegalStateException("Download failed: ${response.code}")
        }

        val body = response.body
        val contentLength = body.contentLength()
        val source = body.source()
        val sink = destinationFile.sink().buffer()

        var totalBytesRead: Long = 0
        val bufferSize = 8L * 1024
        var bytesRead: Long

        try {
            while (source.read(sink.buffer, bufferSize).also { bytesRead = it } != -1L) {
                sink.emit()
                totalBytesRead += bytesRead
                if (downloadState.value == DownloadState.Canceled) {
                    return
                }
                val progress = if (contentLength > 0) {
                    totalBytesRead.toFloat() / contentLength.toFloat()
                } else {
                    -1f
                }
                downloadState.update { DownloadState.Progress(progress) }
            }
        } finally {
            sink.flush()
            sink.close()
            source.close()
        }

        if (downloadState.value == DownloadState.Canceled) {
            runCatching { destinationFile.delete() }
            return
        }

        downloadState.update { DownloadState.Success }
    }

    private fun installApk() {
        val apkFile = getApkFile()
        val apkUri = FileProvider.getUriForFile(context, appFileProvider, apkFile)
        val intent = Intent(Intent.ACTION_VIEW).apply {
            flags = FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_GRANT_READ_URI_PERMISSION
            setDataAndType(apkUri, intentDataType)
        }
        context.startActivity(intent)
    }

    private fun getApkFile(): File {
        return File(context.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS) ?: context.filesDir, "gem.apk")
    }

    fun cancel() {
        downloadState.update { DownloadState.Canceled }
    }
}

private fun isApkStore(store: PlatformStore): Boolean =
    store == PlatformStore.ApkUniversal

sealed interface DownloadState {
    object Idle : DownloadState
    object Preparing : DownloadState
    class Progress(val value: Float) : DownloadState
    object Success : DownloadState
    object Error : DownloadState
    object Canceled : DownloadState
}
