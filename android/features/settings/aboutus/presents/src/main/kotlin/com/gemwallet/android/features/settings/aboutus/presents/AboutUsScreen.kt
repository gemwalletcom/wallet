package com.gemwallet.android.features.settings.aboutus.presents

import android.content.pm.PackageManager
import android.os.Build
import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.settings.aboutus.presents.models.AboutRowUIModel
import com.gemwallet.android.features.settings.aboutus.presents.models.aboutSections
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.listSections
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.open

@Composable
fun AboutUsScreen(
    onCancel: () -> Unit
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    val version = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        context.packageManager.getPackageInfo(context.packageName, PackageManager.PackageInfoFlags.of(0))
    } else {
        context.packageManager.getPackageInfo(context.packageName, 0)
    }.versionName
    val sections = remember(version) { aboutSections(context, version ?: "") }
    Scene(title = stringResource(id = R.string.settings_aboutus), onClose = onCancel) {
        LazyColumn {
            listSections(sections) { position, row ->
                ListItem(
                    model = row.model,
                    listPosition = position,
                    modifier = row.url?.let { url -> Modifier.clickable { uriHandler.open(context, url) } } ?: Modifier,
                    minHeight = ListItemDefaults.plainMinHeight,
                    accessory = row.url?.let { { DataBadgeChevron() } },
                )
            }
        }
    }
}
