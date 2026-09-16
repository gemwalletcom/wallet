package com.gemwallet.android.features.settings.aboutus.presents

import android.content.pm.PackageManager
import android.os.Build
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.toSocialLinks
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.open
import com.gemwallet.android.AppUrl
import uniffi.gemstone.PublicUrl
import uniffi.gemstone.GemAboutRow
import uniffi.gemstone.aboutSections
import uniffi.gemstone.communityLinks
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.lazy.itemsIndexed
import com.gemwallet.android.features.settings.aboutus.presents.localization.stringRes

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
    Scene(title = stringResource(id = R.string.settings_aboutus), onClose = onCancel) {
        LazyColumn {
            aboutSections().forEach { section ->
                itemsIndexed(section.rows) { index, row ->
                    val listPosition = ListPosition.getPosition(index, section.rows.size)
                    when (row) {
                        GemAboutRow.TERMS_OF_SERVICE -> LinkItem(
                            title = stringResource(row.stringRes()),
                            listPosition = listPosition,
                        ) {
                            uriHandler.open(context, AppUrl.page(PublicUrl.TERMS_OF_SERVICE))
                        }
                        GemAboutRow.PRIVACY_POLICY -> LinkItem(
                            title = stringResource(row.stringRes()),
                            listPosition = listPosition,
                        ) {
                            uriHandler.open(context, AppUrl.page(PublicUrl.PRIVACY_POLICY))
                        }
                        GemAboutRow.WEBSITE -> LinkItem(
                            title = stringResource(row.stringRes()),
                            listPosition = listPosition,
                        ) {
                            uriHandler.open(context, AppUrl.page(PublicUrl.WEBSITE))
                        }
                        GemAboutRow.COMMUNITY -> Column {
                            SubheaderItem(row.stringRes())
                            val socials = remember { communityLinks().toSocialLinks() }
                            socials.forEachIndexed { socialIndex, social ->
                                LinkItem(
                                    title = stringResource(id = social.label),
                                    icon = social.icon,
                                    listPosition = ListPosition.getPosition(socialIndex, socials.size),
                                ) {
                                    uriHandler.open(context, social.url)
                                }
                            }
                        }
                        GemAboutRow.VERSION -> LinkItem(
                            title = stringResource(row.stringRes()),
                            listPosition = listPosition,
                            trailingContent = {
                                Text(
                                    text = version ?: "",
                                    textAlign = TextAlign.Center,
                                    style = MaterialTheme.typography.bodyMedium,
                                )
                            },
                            onClick = {},
                        )
                    }
                }
            }
        }
    }
}
