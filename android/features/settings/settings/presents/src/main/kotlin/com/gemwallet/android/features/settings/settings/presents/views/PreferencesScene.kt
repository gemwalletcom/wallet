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
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
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
import com.gemwallet.android.features.settings.settings.viewmodels.models.PreferencesRowAction
import com.gemwallet.android.features.settings.settings.viewmodels.models.preferencesAction
import com.gemwallet.android.features.settings.settings.viewmodels.models.value
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.PreferencesAction
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.compactIconSize
import com.wallet.core.primitives.Appearance
import uniffi.gemstone.GemListRow

@Composable
fun PreferencesScene(
    onAction: (PreferencesAction) -> Unit,
    viewModel: PreferencesViewModel = hiltViewModel(),
) {
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val appearance by viewModel.appearance.collectAsStateWithLifecycle()
    val perpetualDefaults by viewModel.perpetualDefaults.collectAsStateWithLifecycle()
    val configuration = LocalConfiguration.current
    val context = LocalContext.current

    LaunchedEffect(configuration) { viewModel.setLanguage(configuration) }

    Scene(
        title = stringResource(id = (R.string.settings_preferences_title)),
        onClose = { onAction(PreferencesAction.Cancel) },
    ) {
        LazyColumn {
            sections.forEach { section ->
                itemsPositioned(section.rows) { position, row ->
                    when (val action = row.preferencesAction()) {
                        is PreferencesRowAction.Open -> GemListRowView(
                            row = row,
                            listPosition = position,
                            modifier = Modifier.clickable { onAction(action.action) },
                        )
                        PreferencesRowAction.Language -> GemListRowView(
                            row = row,
                            listPosition = position,
                            modifier = Modifier.clickable {
                                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                                    val intent = Intent(Settings.ACTION_APP_LOCALE_SETTINGS)
                                    intent.data = Uri.fromParts("package", context.packageName, null)
                                    context.startActivity(intent)
                                }
                            },
                        )
                        PreferencesRowAction.Appearance -> OptionPickerRow(
                            row = row,
                            listPosition = position,
                            current = appearance,
                            options = Appearance.entries,
                            label = { stringResource(it.stringRes()) },
                            onSelect = { viewModel.setAppearance(it) },
                        )
                        is PreferencesRowAction.Perpetuals -> GemListRowView(
                            row = row,
                            listPosition = position,
                            modifier = Modifier.clickable { viewModel.setPerpetualEnabled(!action.isOn) },
                            onToggle = { _, isOn -> viewModel.setPerpetualEnabled(isOn) },
                        )
                        is PreferencesRowAction.Option -> {
                            val options = viewModel.perpetualOptions.of(action.setting)
                            OptionPickerRow(
                                row = row,
                                listPosition = position,
                                current = perpetualDefaults.value(action.setting),
                                options = options.map { it.value },
                                label = { value -> options.first { it.value == value }.label },
                                onSelect = { viewModel.setPerpetualOption(action.setting, it) },
                            )
                        }
                        null -> GemListRowView(row = row, listPosition = position)
                    }
                }
            }
        }
    }
}

@Composable
private fun <T> OptionPickerRow(
    row: GemListRow,
    listPosition: ListPosition,
    current: T,
    options: List<T>,
    label: @Composable (T) -> String,
    onSelect: (T) -> Unit,
) {
    var expanded by remember { mutableStateOf(false) }
    GemListRowView(
        row = row,
        listPosition = listPosition,
        modifier = if (row is GemListRow.Link) Modifier.clickable { expanded = true } else Modifier,
        onSelect = { expanded = true },
        accessory = {
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
