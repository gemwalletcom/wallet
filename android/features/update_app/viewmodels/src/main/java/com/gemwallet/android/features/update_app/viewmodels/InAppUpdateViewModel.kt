package com.gemwallet.android.features.update_app.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.ext.VersionCheck
import com.gemwallet.android.model.AppUpdateInfo
import com.gemwallet.android.model.BuildInfo
import com.wallet.core.primitives.PlatformStore
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class InAppUpdateViewModel @Inject constructor(
    private val buildInfo: BuildInfo,
    private val userConfig: UserConfig,
    private val updateService: InAppUpdateService,
) : ViewModel() {

    val updateAvailable = userConfig.getAppVersionSkip()
        .combine(userConfig.getLatestAppUpdate()) { skip, update ->
            availableUpdate(update, skip)
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    internal fun availableUpdate(
        update: AppUpdateInfo?,
        skipVersion: String,
    ): AppUpdateInfo? = update?.takeIf {
        VersionCheck.isVersionHigher(new = it.version, current = buildInfo.versionName)
            && buildInfo.platformStore == PlatformStore.ApkUniversal
            && (it.isRequired || it.version != skipVersion)
    }

    private val _downloadState = MutableStateFlow<DownloadState>(DownloadState.Idle)
    val downloadState = _downloadState.asStateFlow()
    private var downloadJob: Job? = null

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

        downloadJob = viewModelScope.launch {
            _downloadState.value = DownloadState.Preparing
            try {
                updateService.download(update.version) { progress ->
                    _downloadState.value = DownloadState.Progress(progress)
                }
                tryInstall()
            } catch (_: CancellationException) {
                _downloadState.value = DownloadState.Canceled
            } catch (_: Throwable) {
                _downloadState.value = DownloadState.Error
            } finally {
                downloadJob = null
            }
        }
    }

    private fun tryInstall() {
        val update = updateAvailable.value ?: return
        if (!updateService.canRequestPackageInstalls()) {
            _downloadState.value = DownloadState.PermissionRequired
            return
        }
        viewModelScope.launch {
            try {
                updateService.installDownloadedUpdate(update.version)
                _downloadState.value = DownloadState.Success
            } catch (_: Throwable) {
                _downloadState.value = DownloadState.Error
            }
        }
    }

    fun dismissPermissionPrompt() {
        _downloadState.value = DownloadState.Idle
    }

    fun skip() {
        val update = updateAvailable.value ?: return
        if (update.isRequired) {
            return
        }
        viewModelScope.launch {
            userConfig.setAppVersionSkip(update.version)
        }
    }

    fun cancel() {
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
