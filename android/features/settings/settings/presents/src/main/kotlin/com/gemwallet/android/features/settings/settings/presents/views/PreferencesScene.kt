package com.gemwallet.android.features.settings.settings.presents.views

import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
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
import com.gemwallet.android.domains.perpetual.PerpetualConfig
import com.gemwallet.android.domains.perpetual.formatLeverage
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
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
    val perpetualTakeProfit by viewModel.perpetualTakeProfit.collectAsStateWithLifecycle()
    val perpetualStopLoss by viewModel.perpetualStopLoss.collectAsStateWithLifecycle()

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

            if (isPerpetualEnabled) {
                item {
                    OptionPickerLinkItem(
                        title = stringResource(R.string.settings_preferences_default_leverage),
                        current = perpetualLeverage,
                        options = PerpetualConfig.leverageOptions,
                        listPosition = ListPosition.Middle,
                        label = { it.formatLeverage() },
                        onSelect = { viewModel.setPerpetualLeverage(it) },
                    )
                }
                item {
                    OptionPickerLinkItem(
                        title = stringResource(R.string.settings_preferences_default_take_profit),
                        current = perpetualTakeProfit,
                        options = PerpetualConfig.takeProfitOptions,
                        listPosition = ListPosition.Middle,
                        label = { autocloseLabel(it) },
                        onSelect = { viewModel.setPerpetualTakeProfit(it) },
                    )
                }
                item {
                    OptionPickerLinkItem(
                        title = stringResource(R.string.settings_preferences_default_stop_loss),
                        current = perpetualStopLoss,
                        options = PerpetualConfig.stopLossOptions,
                        listPosition = ListPosition.Last,
                        label = { autocloseLabel(it) },
                        onSelect = { viewModel.setPerpetualStopLoss(it) },
                    )
                }
            }
        }
    }
}

@Composable
private fun autocloseLabel(percent: Int): String =
    if (percent == 0) stringResource(R.string.common_none) else "$percent%"

@Composable
private fun <T> OptionPickerLinkItem(
    title: String,
    current: T,
    options: List<T>,
    listPosition: ListPosition,
    label: @Composable (T) -> String,
    onSelect: (T) -> Unit,
) {
    var expanded by remember { mutableStateOf(false) }
    LinkItem(
        title = title,
        listPosition = listPosition,
        indented = true,
        trailingContent = {
            PropertyDataText(text = label(current), badge = { DataBadgeChevron() })
            DropdownMenu(
                expanded = expanded,
                onDismissRequest = { expanded = false },
                containerColor = MaterialTheme.colorScheme.background,
            ) {
                options.forEach { option ->
                    DropdownMenuItem(
                        text = {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                if (option == current) {
                                    Icon(AppIcons.Check, null, modifier = Modifier.size(compactIconSize))
                                } else {
                                    Spacer(modifier = Modifier.size(compactIconSize))
                                }
                                Spacer4()
                                Text(label(option))
                            }
                        },
                        onClick = {
                            onSelect(option)
                            expanded = false
                        },
                    )
                }
            }
        },
        onClick = { expanded = true },
    )
}
