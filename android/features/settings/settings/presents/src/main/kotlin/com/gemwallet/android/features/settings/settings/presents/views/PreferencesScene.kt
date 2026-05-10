package com.gemwallet.android.features.settings.settings.presents.views

import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.features.settings.currency.presents.components.emojiFlags
import com.gemwallet.android.features.settings.settings.viewmodels.SettingsViewModel
import java.util.Locale

@Composable
fun PreferencesScene(
    onCurrencies: () -> Unit,
    onNetworks: () -> Unit,
    onCancel: () -> Unit,
    viewModel: SettingsViewModel = hiltViewModel(),
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val isPerpetualEnabled by viewModel.isPerpetualEnabled.collectAsStateWithLifecycle()
    val perpetualLeverage by viewModel.perpetualLeverage.collectAsStateWithLifecycle()

    val configuration = LocalConfiguration.current
    val context = LocalContext.current

    var showLeveragePicker by remember { mutableStateOf(false) }

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

            if (uiState.developEnabled) {
                item {
                    LinkItem(
                        title = stringResource(id = R.string.perpetuals_title),
                        icon = R.drawable.settings_pricealert,
                        listPosition = if (isPerpetualEnabled) ListPosition.First else ListPosition.Single,
                        trailingContent = {
                            Switch(
                                checked = isPerpetualEnabled,
                                onCheckedChange = viewModel::setPerpetualEnabled,
                            )
                        },
                        onClick = { viewModel.setPerpetualEnabled(!isPerpetualEnabled) },
                    )
                }
            }

            if (uiState.developEnabled && isPerpetualEnabled) {
                item {
                    LinkItem(
                        title = stringResource(id = R.string.settings_preferences_default_leverage),
                        listPosition = ListPosition.Last,
                        trailingContent = {
                            PropertyDataText(
                                text = "${perpetualLeverage}x",
                                badge = { DataBadgeChevron() },
                            )
                            DropdownMenu(
                                expanded = showLeveragePicker,
                                onDismissRequest = { showLeveragePicker = false },
                                containerColor = MaterialTheme.colorScheme.background,
                            ) {
                                viewModel.perpetualLeverageOptions.forEach { value ->
                                    DropdownMenuItem(
                                        text = {
                                            Row(verticalAlignment = Alignment.CenterVertically) {
                                                if (value == perpetualLeverage) {
                                                    Icon(Icons.Default.Check, null, modifier = Modifier.size(compactIconSize))
                                                } else {
                                                    Spacer(modifier = Modifier.size(compactIconSize))
                                                }
                                                Spacer4()
                                                Text("${value}x")
                                            }
                                        },
                                        onClick = {
                                            viewModel.setPerpetualLeverage(value)
                                            showLeveragePicker = false
                                        },
                                    )
                                }
                            }
                        },
                        onClick = { showLeveragePicker = true },
                    )
                }
            }
        }
    }
}
