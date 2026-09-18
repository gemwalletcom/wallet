package com.gemwallet.android.features.settings.aboutus.presents

import android.content.pm.PackageManager
import android.os.Build
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.gemListSections
import com.gemwallet.android.ui.components.screen.Scene
import uniffi.gemstone.aboutSections

@Composable
fun AboutUsScreen(
    onCancel: () -> Unit
) {
    val context = LocalContext.current
    val version = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        context.packageManager.getPackageInfo(context.packageName, PackageManager.PackageInfoFlags.of(0))
    } else {
        context.packageManager.getPackageInfo(context.packageName, 0)
    }.versionName
    val sections = remember(version) { aboutSections(version ?: "", update = null) }
    Scene(title = stringResource(id = R.string.settings_aboutus), onClose = onCancel) {
        LazyColumn {
            gemListSections(sections)
        }
    }
}
