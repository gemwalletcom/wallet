package com.gemwallet.android.features.settings.settings.presents.views

import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.features.settings.currency.presents.components.emojiFlags
import com.gemwallet.android.features.settings.settings.viewmodels.SettingsViewModel
import java.util.Locale

@Composable
fun PreferencesScene(
    onCurrencies: () -> Unit,
    onNetworks: () -> Unit,
    onPerpetual: () -> Unit,
    onCancel: () -> Unit,
    viewModel: SettingsViewModel = hiltViewModel(),
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    val configuration = LocalConfiguration.current
    val context = LocalContext.current

    Scene(
        title = stringResource(id = (R.string.settings_preferences_title)),
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                LinkItem(
                    title = stringResource(R.string.settings_currency),
                    icon = R.drawable.settings_currency,
                    listPosition = ListPosition.First,
                    trailingContent = {
                        PropertyDataText(
                            text = "${emojiFlags[uiState.currency.string]}  ${uiState.currency.string}",
                            badge = { DataBadgeChevron() },
                        )
                    },
                    onClick = onCurrencies,
                )
            }

            item {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                    val language = configuration.locales
                        .get(0).displayLanguage.replaceFirstChar {
                            if (it.isLowerCase()) it.titlecase(Locale.ROOT) else it.toString()
                        }
                    LinkItem(
                        title = stringResource(id = R.string.settings_language),
                        icon = R.drawable.settings_language,
                        trailingContent = {
                            PropertyDataText(
                                text = language,
                                badge = { DataBadgeChevron() },
                            )
                        },
                        onClick = {
                            val intent = Intent(Settings.ACTION_APP_LOCALE_SETTINGS)
                            intent.data = Uri.fromParts("package", context.packageName, null)
                            context.startActivity(intent)
                        }
                    )
                }
            }

            item {
                LinkItem(
                    title = stringResource(id = R.string.settings_networks_title),
                    icon = R.drawable.settings_networks,
                    listPosition = ListPosition.Last
                ) {
                    onNetworks()
                }
            }
        }
    }
}
