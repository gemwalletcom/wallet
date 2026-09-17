package com.gemwallet.android.features.settings.settings.presents.views

import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
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
import com.gemwallet.android.features.settings.settings.viewmodels.PreferencesViewModel
import com.gemwallet.android.features.settings.settings.viewmodels.localization.stringRes
import com.gemwallet.android.features.settings.settings.viewmodels.models.PreferencesRowUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.PreferencesAction
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.compactIconSize
import com.wallet.core.primitives.Appearance
import java.util.Locale

@Composable
fun PreferencesScene(
    onAction: (PreferencesAction) -> Unit,
    viewModel: PreferencesViewModel = hiltViewModel(),
) {
    val rows by viewModel.rows.collectAsStateWithLifecycle()
    val configuration = LocalConfiguration.current
    val context = LocalContext.current

    Scene(
        title = stringResource(id = (R.string.settings_preferences_title)),
        onClose = { onAction(PreferencesAction.Cancel) },
    ) {
        LazyColumn {
            rows.forEach { section ->
                itemsIndexed(section) { index, row ->
                    val listPosition = ListPosition.getPosition(index, section.size)
                    when (row) {
                        is PreferencesRowUIModel.Link -> ListItem(
                            model = row.model,
                            listPosition = listPosition,
                            modifier = Modifier.clickable { onAction(row.action) },
                            minHeight = ListItemDefaults.plainMinHeight,
                            accessory = { DataBadgeChevron() },
                        )
                        is PreferencesRowUIModel.Language -> if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                            val language = configuration.locales
                                .get(0).displayLanguage.replaceFirstChar {
                                    if (it.isLowerCase()) it.titlecase(Locale.ROOT) else it.toString()
                                }
                            ListItem(
                                model = row.model.copy(subtitle = language),
                                listPosition = listPosition,
                                modifier = Modifier.clickable {
                                    val intent = Intent(Settings.ACTION_APP_LOCALE_SETTINGS)
                                    intent.data = Uri.fromParts("package", context.packageName, null)
                                    context.startActivity(intent)
                                },
                                minHeight = ListItemDefaults.plainMinHeight,
                                accessory = { DataBadgeChevron() },
                            )
                        }
                        is PreferencesRowUIModel.AppearancePicker -> OptionPickerLinkItem(
                            model = row.model,
                            current = row.current,
                            options = Appearance.entries,
                            listPosition = listPosition,
                            label = { stringResource(it.stringRes()) },
                            onSelect = { viewModel.setAppearance(it) },
                        )
                        is PreferencesRowUIModel.PerpetualsSwitch -> ListItem(
                            model = row.model,
                            listPosition = listPosition,
                            modifier = Modifier.clickable { viewModel.setPerpetualEnabled(!row.isEnabled) },
                            minHeight = ListItemDefaults.plainMinHeight,
                            accessory = {
                                Switch(
                                    checked = row.isEnabled,
                                    onCheckedChange = viewModel::setPerpetualEnabled,
                                )
                            },
                        )
                        is PreferencesRowUIModel.Picker -> OptionPickerLinkItem(
                            model = row.model,
                            current = row.current,
                            options = row.options.map { it.value },
                            listPosition = listPosition,
                            label = { value -> row.options.first { it.value == value }.label },
                            onSelect = { viewModel.setPerpetualOption(row.setting, it) },
                        )
                    }
                }
            }
        }
    }
}
@Composable
private fun <T> OptionPickerLinkItem(
    model: ListItemModel,
    current: T,
    options: List<T>,
    listPosition: ListPosition,
    label: @Composable (T) -> String,
    onSelect: (T) -> Unit,
) {
    var expanded by remember { mutableStateOf(false) }
    ListItem(
        model = model,
        listPosition = listPosition,
        modifier = Modifier.clickable { expanded = true },
        accessory = {
            DataBadgeChevron()
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
    )
}
