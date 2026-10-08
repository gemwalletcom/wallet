package com.gemwallet.android.features.settings.presents

import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.settings.viewmodels.models.PerpetualSetting
import com.gemwallet.android.features.settings.viewmodels.models.PreferencesRowAction
import com.gemwallet.android.features.settings.viewmodels.models.of
import com.gemwallet.android.features.settings.viewmodels.models.preferencesAction
import com.gemwallet.android.features.settings.viewmodels.models.value
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.OptionPickerRow
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.actions.PreferencesAction
import com.wallet.core.primitives.Appearance
import uniffi.gemstone.GemListSection
import uniffi.gemstone.GemPerpetualDefaults
import uniffi.gemstone.GemPerpetualPickers

@Composable
fun PreferencesScene(
    sections: List<GemListSection>,
    appearance: Appearance,
    perpetualDefaults: GemPerpetualDefaults,
    perpetualOptions: GemPerpetualPickers,
    onAction: (PreferencesAction) -> Unit,
    onAppearance: (Appearance) -> Unit,
    onPerpetualEnabled: (Boolean) -> Unit,
    onPerpetualOption: (PerpetualSetting, Int) -> Unit,
) {
    val context = LocalContext.current

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
                            onSelect = { onAppearance(it) },
                        )

                        is PreferencesRowAction.Perpetuals -> GemListRowView(
                            row = row,
                            listPosition = position,
                            modifier = Modifier.clickable { onPerpetualEnabled(!action.isOn) },
                            onToggle = { _, isOn -> onPerpetualEnabled(isOn) },
                        )

                        is PreferencesRowAction.Option -> {
                            val options = perpetualOptions.of(action.setting)
                            OptionPickerRow(
                                row = row,
                                listPosition = position,
                                current = perpetualDefaults.value(action.setting),
                                options = options.map { it.value.toInt() },
                                label = { value -> options.first { it.value.toInt() == value }.label.string(LocalContext.current) },
                                onSelect = { onPerpetualOption(action.setting, it) },
                            )
                        }

                        null -> GemListRowView(row = row, listPosition = position)
                    }
                }
            }
        }
    }
}
