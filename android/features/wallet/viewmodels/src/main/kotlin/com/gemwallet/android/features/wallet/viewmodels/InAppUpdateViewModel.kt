package com.gemwallet.android.features.wallet.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.update.cases.ObserveAppUpdateOffer
import com.gemwallet.android.application.update.cases.SkipAppUpdate
import com.gemwallet.android.ext.runCatchingCancellable
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class InAppUpdateViewModel @Inject constructor(observeAppUpdateOffer: ObserveAppUpdateOffer, private val skipAppUpdate: SkipAppUpdate, private val updateService: InAppUpdateService) : ViewModel() {

    val updateAvailable = observeAppUpdateOffer.observeAppUpdateOffer()
        .map { offer -> offer?.takeIf { it.apkUrl != null } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val _downloadState = MutableStateFlow<DownloadState>(DownloadState.Idle)
    val downloadState = _downloadState.asStateFlow()
    private var downloadJob: Job? = null
    private var downloadedVersion: String? = null

    init {
        viewModelScope.launch {
            updateService.clearDownloadedUpdate()
        }
    }

    fun update() {
        if (downloadJob?.isActive == true) return
        if (_downloadState.value == DownloadState.PermissionRequired) {
            tryInstall()
            return
        }

        val update = updateAvailable.value ?: return
        val apkUrl = update.apkUrl ?: return

        downloadJob = viewModelScope.launch {
            downloadedVersion = null
            _downloadState.value = DownloadState.Preparing
            try {
                updateService.download(apkUrl, update.version) { progress ->
                    _downloadState.value = DownloadState.Progress(progress)
                }
                downloadedVersion = update.version
                tryInstall()
            } catch (_: CancellationException) {
                downloadedVersion = null
                _downloadState.value = DownloadState.Canceled
            } catch (_: Throwable) {
                downloadedVersion = null
                _downloadState.value = DownloadState.Error
            } finally {
                downloadJob = null
            }
        }
    }

    private fun tryInstall() {
        val update = updateAvailable.value
        if (update == null || downloadedVersion != update.version) {
            downloadedVersion = null
            _downloadState.value = DownloadState.Error
            return
        }
        if (!updateService.canRequestPackageInstalls()) {
            _downloadState.value = DownloadState.PermissionRequired
            return
        }
        _downloadState.value = DownloadState.Preparing
        viewModelScope.launch {
            try {
                updateService.installDownloadedUpdate(update.version)
                _downloadState.value = DownloadState.Success
            } catch (cancellation: CancellationException) {
                throw cancellation
            } catch (_: Throwable) {
                _downloadState.value = DownloadState.Error
            }
        }
    }

    fun onInstallPermissionResult() {
        if (_downloadState.value != DownloadState.PermissionRequired || !updateService.canRequestPackageInstalls()) return
        tryInstall()
    }

    fun dismissPermissionPrompt() {
        downloadedVersion = null
        _downloadState.value = DownloadState.Idle
    }

    fun skip() {
        val update = updateAvailable.value ?: return
        viewModelScope.launch {
            runCatchingCancellable { skipAppUpdate.skipAppUpdate(update) }
                .onFailure { Log.e(TAG, "skipping update ${update.version} failed", it) }
        }
    }

    fun cancel() {
        downloadedVersion = null
        downloadJob?.cancel()
        updateService.cancel()
    }
}

sealed interface DownloadState {
    object Idle : DownloadState
    object Preparing : DownloadState
    data class Progress(val fraction: Float?) : DownloadState
    object PermissionRequired : DownloadState
    object Success : DownloadState
    object Error : DownloadState
    object Canceled : DownloadState
}

private const val TAG = "InAppUpdate"
