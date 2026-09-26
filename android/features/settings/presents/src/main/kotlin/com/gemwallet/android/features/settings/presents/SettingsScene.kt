@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.settings.presents

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.settings.viewmodels.models.settingsAction
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.actions.SettingsAction
import com.gemwallet.android.ui.theme.space0
import uniffi.gemstone.GemListSection

@Composable
fun SettingsScene(sections: List<GemListSection>, onAction: (SettingsAction) -> Unit, scrollState: ScrollState) {
    Scene(
        title = stringResource(id = R.string.settings_title),
        mainActionPadding = PaddingValues(space0),
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(scrollState),
        ) {
            sections.forEach { section ->
                section.rows.forEachIndexed { index, row ->
                    val action = row.settingsAction()
                    GemListRowView(
                        row = row,
                        listPosition = ListPosition.getPosition(index, section.rows.size),
                        modifier = Modifier.clickable { action?.let(onAction) },
                    )
                }
            }
            Spacer(modifier = Modifier.size(it.calculateBottomPadding()))
        }
    }
}
