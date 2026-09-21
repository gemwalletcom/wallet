package com.gemwallet.android.features.settings.aboutus.viewmodels

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import androidx.lifecycle.ViewModel
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.ui.R
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemListSection
import uniffi.gemstone.aboutSections
import javax.inject.Inject

@HiltViewModel
class AboutUsViewModel @Inject constructor(private val userConfig: UserConfig, @param:ApplicationContext private val context: Context) : ViewModel() {

    val sections: List<GemListSection> = context.packageInfo().let {
        aboutSections(version = it.first, build = it.second, update = null)
    }

    private val developerEnabled = MutableStateFlow(userConfig.developEnabled())
    val isDeveloperEnabled: StateFlow<Boolean> = developerEnabled.asStateFlow()

    fun developerMenuTitle(isEnabled: Boolean): String = context.getString(
        if (isEnabled) R.string.settings_disable_value else R.string.settings_enable_value,
        context.getString(R.string.settings_developer),
    )

    fun toggleDeveloperMode() {
        userConfig.developEnabled(!userConfig.developEnabled())
        developerEnabled.update { userConfig.developEnabled() }
    }
}

fun GemListRow.opensDeveloperMenu(): Boolean = this is GemListRow.Text && title == GemListRowTitle.VERSION

private fun Context.packageInfo(): Pair<String, String> {
    val info = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        packageManager.getPackageInfo(packageName, PackageManager.PackageInfoFlags.of(0))
    } else {
        packageManager.getPackageInfo(packageName, 0)
    }
    return (info.versionName ?: "") to info.longVersionCode.toString()
}
