package com.gemwallet.android.features.settings.settings.presents.views

import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.annotation.DrawableRes
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
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.perpetual.formatLeverage
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import androidx.compose.foundation.lazy.itemsIndexed
import com.gemwallet.android.features.settings.settings.presents.localization.stringRes
import com.gemwallet.android.features.settings.settings.presents.style.icon
import com.gemwallet.android.features.settings.settings.presents.style.painter
import uniffi.gemstone.GemPreferencesRow
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.features.settings.settings.viewmodels.PreferencesViewModel
import com.wallet.core.primitives.Appearance
import java.util.Locale
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.PerpetualProvider
import com.gemwallet.android.math.toUnsignedInts

@Composable
fun PreferencesScene(
    onAction: (PreferencesAction) -> Unit,
    viewModel: PreferencesViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val isPerpetualEnabled by viewModel.isPerpetualEnabled.collectAsStateWithLifecycle()
    val appearance by viewModel.appearance.collectAsStateWithLifecycle()
    val perpetualLeverage by viewModel.perpetualLeverage.collectAsStateWithLifecycle()
    val perpetualTakeProfit by viewModel.perpetualTakeProfit.collectAsStateWithLifecycle()
    val perpetualStopLoss by viewModel.perpetualStopLoss.collectAsStateWithLifecycle()

    val configuration = LocalConfiguration.current
    val context = LocalContext.current

    Scene(
        title = stringResource(id = (R.string.settings_preferences_title)),
        onClose = { onAction(PreferencesAction.Cancel) },
    ) {
        LazyColumn {
            state.sections.forEach { section ->
                itemsIndexed(section.rows) { index, row ->
                    val listPosition = ListPosition.getPosition(index, section.rows.size)
                    when (row) {
                        GemPreferencesRow.CURRENCY -> LinkItem(
                            title = stringResource(row.stringRes()),
                            painter = row.painter(),
                            listPosition = listPosition,
                            trailingContent = {
                                PropertyDataText(
                                    text = state.currency.text(),
                                    badge = { DataBadgeChevron() },
                                )
                            },
                            onClick = { onAction(PreferencesAction.Currencies) },
                        )
                        GemPreferencesRow.LANGUAGE -> if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                            val language = configuration.locales
                                .get(0).displayLanguage.replaceFirstChar {
                                    if (it.isLowerCase()) it.titlecase(Locale.ROOT) else it.toString()
                                }
                            LinkItem(
                                title = stringResource(row.stringRes()),
                                painter = row.painter(),
                                listPosition = listPosition,
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
                        GemPreferencesRow.APPEARANCE -> OptionPickerLinkItem(
                            title = stringResource(row.stringRes()),
                            current = appearance,
                            options = Appearance.entries,
                            listPosition = listPosition,
                            icon = row.icon(),
                            indented = false,
                            label = { stringResource(it.stringRes()) },
                            onSelect = { viewModel.setAppearance(it) },
                        )
                        GemPreferencesRow.NETWORKS -> LinkItem(
                            title = stringResource(row.stringRes()),
                            painter = row.painter(),
                            listPosition = listPosition,
                        ) {
                            onAction(PreferencesAction.Networks)
                        }
                        GemPreferencesRow.CONTACTS -> LinkItem(
                            title = stringResource(row.stringRes()),
                            painter = row.painter(),
                            listPosition = listPosition,
                            onClick = { onAction(PreferencesAction.Contacts) },
                        )
                        GemPreferencesRow.PERPETUALS -> LinkItem(
                            title = stringResource(row.stringRes()),
                            painter = row.painter(),
                            listPosition = listPosition,
                            trailingContent = {
                                Switch(
                                    checked = isPerpetualEnabled,
                                    onCheckedChange = viewModel::setPerpetualEnabled,
                                )
                            },
                            onClick = { viewModel.setPerpetualEnabled(!isPerpetualEnabled) },
                        )
                        GemPreferencesRow.PERPETUAL_LEVERAGE -> OptionPickerLinkItem(
                            title = stringResource(row.stringRes()),
                            current = perpetualLeverage,
                            options = GemPerpetual(PerpetualProvider.HYPERCORE).use { it.leverageOptions(null) }.toUnsignedInts(),
                            listPosition = listPosition,
                            label = { it.formatLeverage() },
                            onSelect = { viewModel.setPerpetualLeverage(it) },
                        )
                        GemPreferencesRow.PERPETUAL_TAKE_PROFIT -> OptionPickerLinkItem(
                            title = stringResource(row.stringRes()),
                            current = perpetualTakeProfit,
                            options = GemPerpetual(PerpetualProvider.HYPERCORE).use { it.takeProfitOptions() }.toUnsignedInts(),
                            listPosition = listPosition,
                            label = { autocloseLabel(it) },
                            onSelect = { viewModel.setPerpetualTakeProfit(it) },
                        )
                        GemPreferencesRow.PERPETUAL_STOP_LOSS -> OptionPickerLinkItem(
                            title = stringResource(row.stringRes()),
                            current = perpetualStopLoss,
                            options = GemPerpetual(PerpetualProvider.HYPERCORE).use { it.stopLossOptions() }.toUnsignedInts(),
                            listPosition = listPosition,
                            label = { autocloseLabel(it) },
                            onSelect = { viewModel.setPerpetualStopLoss(it) },
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun autocloseLabel(percent: Int): String =
    GemPerpetual(PerpetualProvider.HYPERCORE).use { it.autoclosePercent(percent.toUByte()) }
        ?.let { "$it%" } ?: stringResource(R.string.common_none)

@Composable
private fun <T> OptionPickerLinkItem(
    title: String,
    current: T,
    options: List<T>,
    listPosition: ListPosition,
    label: @Composable (T) -> String,
    onSelect: (T) -> Unit,
    @DrawableRes icon: Int? = null,
    indented: Boolean = true,
) {
    var expanded by remember { mutableStateOf(false) }
    LinkItem(
        title = title,
        painter = icon?.let { painterResource(id = it) },
        listPosition = listPosition,
        indented = indented,
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
