package com.gemwallet.android.features.update_app.viewmodels

import android.content.Context
import android.content.Intent
import android.content.Intent.FLAG_ACTIVITY_NEW_TASK
import android.content.pm.PackageInfo
import android.content.pm.PackageManager
import android.content.pm.Signature
import android.os.Build
import android.os.Environment
import androidx.core.content.FileProvider
import com.gemwallet.android.ext.VersionCheck
import com.gemwallet.android.ext.universalApkDownloadUrl
import com.gemwallet.android.model.BuildInfo
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.isActive
import kotlinx.coroutines.withContext
import okhttp3.Call
import okhttp3.OkHttpClient
import okhttp3.Request
import okio.buffer
import okio.sink
import java.io.File
import java.security.MessageDigest
import java.util.concurrent.TimeUnit
import javax.inject.Inject

class InAppUpdateServiceImpl @Inject constructor(
    @param:ApplicationContext private val context: Context,
    private val buildInfo: BuildInfo,
) : InAppUpdateService {

    private val appFileProvider = "${context.packageName}.provider"
    private val intentDataType = "application/vnd.android.package-archive"
    private val client = OkHttpClient.Builder()
        .connectTimeout(30, TimeUnit.SECONDS)
        .readTimeout(60, TimeUnit.SECONDS)
        .build()

    @Volatile
    private var currentCall: Call? = null

    override fun canRequestPackageInstalls(): Boolean =
        context.packageManager.canRequestPackageInstalls()

    override suspend fun clearDownloadedUpdate() {
        withContext(Dispatchers.IO) {
            deleteDownloadedApk()
        }
    }

    override suspend fun download(version: String, onProgress: (Float?) -> Unit): Unit = withContext(Dispatchers.IO) {
        val destinationFile = getApkFile()
        val coroutineContext = currentCoroutineContext()
        deleteDownloadedApk()

        try {
            val request = Request.Builder()
                .url(universalApkDownloadUrl(version))
                .build()
            val call = client.newCall(request)
            currentCall = call
            coroutineContext.ensureActive()
            bindCancellation(call, coroutineContext[Job])

            call.execute().use { response ->
                if (!response.isSuccessful) {
                    throw IllegalStateException("Download failed: ${response.code}")
                }

                writeDownloadedApk(
                    destinationFile = destinationFile,
                    responseBody = response.body,
                    onProgress = onProgress,
                    coroutineContext = coroutineContext,
                )
            }

            validateDownloadedApk(destinationFile, version)
        } catch (cancellation: CancellationException) {
            deleteDownloadedApk()
            throw cancellation
        } catch (throwable: Throwable) {
            deleteDownloadedApk()
            if (!coroutineContext.isActive) {
                throw CancellationException("Update download canceled").apply {
                    initCause(throwable)
                }
            }
            throw throwable
        } finally {
            currentCall = null
        }
    }

    override fun installDownloadedUpdate(version: String) {
        val apkFile = getApkFile()

        if (!apkFile.exists()) {
            throw IllegalStateException("Downloaded update missing")
        }

        validateDownloadedApk(apkFile, version)

        val apkUri = FileProvider.getUriForFile(context, appFileProvider, apkFile)
        val intent = Intent(Intent.ACTION_VIEW).apply {
            flags = FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_GRANT_READ_URI_PERMISSION
            setDataAndType(apkUri, intentDataType)
        }
        context.startActivity(intent)
    }

    override fun cancel() {
        currentCall?.cancel()
    }

    private fun bindCancellation(call: Call, job: Job?) {
        job?.invokeOnCompletion { cause ->
            if (cause is CancellationException) {
                call.cancel()
            }
        }
    }

    private fun writeDownloadedApk(
        destinationFile: File,
        responseBody: okhttp3.ResponseBody,
        onProgress: (Float?) -> Unit,
        coroutineContext: kotlin.coroutines.CoroutineContext,
    ) {
        val contentLength = responseBody.contentLength()

        responseBody.source().use { source ->
            destinationFile.sink().buffer().use { sink ->
                var totalBytesRead = 0L
                val bufferSize = 8L * 1024

                while (true) {
                    coroutineContext.ensureActive()
                    val bytesRead = source.read(sink.buffer, bufferSize)
                    if (bytesRead == -1L) {
                        break
                    }
                    sink.emit()
                    totalBytesRead += bytesRead
                    onProgress(
                        if (contentLength > 0) {
                            totalBytesRead.toFloat() / contentLength.toFloat()
                        } else {
                            null
                        }
                    )
                }
            }
        }
    }

    private fun validateDownloadedApk(apkFile: File, expectedVersion: String) {
        val packageManager = context.packageManager
        val archiveInfo = packageManager.getArchivePackageInfo(apkFile)
            ?: throw IllegalStateException("Downloaded APK metadata unavailable")
        val archiveVersion = archiveInfo.versionName
            ?: throw IllegalStateException("Downloaded APK version unavailable")

        if (archiveInfo.packageName != context.packageName) {
            throw IllegalStateException("Downloaded APK package mismatch")
        }
        if (archiveVersion != expectedVersion) {
            throw IllegalStateException("Downloaded APK version mismatch")
        }
        if (!VersionCheck.isVersionHigher(new = archiveVersion, current = buildInfo.versionName)) {
            throw IllegalStateException("Downloaded APK is not a newer version")
        }

        val installedSigners = packageManager.getInstalledPackageInfo(context.packageName).signerDigests()
        val archiveSigners = archiveInfo.signerDigests()
        if (installedSigners.isEmpty() || archiveSigners.isEmpty() || installedSigners != archiveSigners) {
            throw IllegalStateException("Downloaded APK signer mismatch")
        }
    }

    private fun PackageManager.getInstalledPackageInfo(packageName: String): PackageInfo =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            getPackageInfo(packageName, PackageManager.PackageInfoFlags.of(PackageManager.GET_SIGNING_CERTIFICATES.toLong()))
        } else {
            @Suppress("DEPRECATION")
            getPackageInfo(packageName, PackageManager.GET_SIGNING_CERTIFICATES)
        }

    private fun PackageManager.getArchivePackageInfo(apkFile: File): PackageInfo? =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            getPackageArchiveInfo(apkFile.path, PackageManager.PackageInfoFlags.of(PackageManager.GET_SIGNING_CERTIFICATES.toLong()))
        } else {
            @Suppress("DEPRECATION")
            getPackageArchiveInfo(apkFile.path, PackageManager.GET_SIGNING_CERTIFICATES)
        }

    private fun PackageInfo.signerDigests(): Set<String> =
        signingInfo?.apkContentsSigners
            .orEmpty()
            .map { signature -> signature.sha256Digest() }
            .toSet()

    private fun Signature.sha256Digest(): String =
        MessageDigest.getInstance("SHA-256")
            .digest(toByteArray())
            .joinToString(separator = "") { byte -> "%02x".format(byte) }

    private fun deleteDownloadedApk() {
        runCatching { getApkFile().delete() }
    }

    private fun getApkFile(): File =
        File(
            context.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS) ?: context.filesDir,
            APK_FILE_NAME,
        )

    companion object {
        private const val APK_FILE_NAME = "gem.apk"
    }
}
