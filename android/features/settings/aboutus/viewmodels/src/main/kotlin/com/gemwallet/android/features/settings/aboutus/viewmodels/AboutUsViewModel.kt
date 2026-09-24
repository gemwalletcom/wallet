package com.gemwallet.android.features.settings.aboutus.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.BuildInfo
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAboutViewState
import uniffi.gemstone.GemAppUpdateServiceInterface
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.Release
import uniffi.gemstone.aboutViewState
import javax.inject.Inject

@HiltViewModel
class AboutUsViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val buildInfo: BuildInfo,
    private val appUpdateService: GemAppUpdateServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val release = MutableStateFlow<Release?>(null)
    private val developerEnabled = MutableStateFlow(userConfig.developEnabled())

    val viewState: StateFlow<GemAboutViewState> = combine(release, developerEnabled, ::viewState)
        .stateIn(viewModelScope, SharingStarted.Eagerly, viewState(null, developerEnabled.value))

    init {
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { appUpdateService.newest(buildInfo.platformStore.toGem(), buildInfo.versionName) }
                .onSuccess { release.value = it }
                .onFailure { Log.e(TAG, "newest release failed", it) }
        }
    }

    fun toggleDeveloperMode() {
        userConfig.developEnabled(!userConfig.developEnabled())
        developerEnabled.update { userConfig.developEnabled() }
    }

    private fun viewState(release: Release?, developerEnabled: Boolean): GemAboutViewState = aboutViewState(
        version = buildInfo.versionName,
        build = buildInfo.versionCode.toString(),
        update = release,
        developerEnabled = developerEnabled,
    )

    private companion object {
        const val TAG = "AboutUsViewModel"
    }
}

fun GemListRow.opensDeveloperMenu(): Boolean = this is GemListRow.Text && title == GemListRowTitle.VERSION
