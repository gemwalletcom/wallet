package com.gemwallet.android.features.settings.aboutus.presents

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.aboutus.viewmodels.AboutUsViewModel
import com.gemwallet.android.features.settings.aboutus.viewmodels.opensDeveloperMenu
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.titleRes

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun AboutUsScreen(onCancel: () -> Unit, viewModel: AboutUsViewModel = hiltViewModel()) {
    val context = LocalContext.current
    val viewState by viewModel.viewState.collectAsStateWithLifecycle()
    var isShowDevelopEnable by remember { mutableStateOf(false) }

    Scene(title = stringResource(id = R.string.settings_aboutus), onClose = onCancel) {
        LazyColumn {
            viewState.sections.forEachIndexed { index, section ->
                section.title?.titleRes()?.let { title -> item(key = "section:$index") { SubheaderItem(title) } }
                itemsPositioned(section.rows) { position, row ->
                    val opensMenu = row.opensDeveloperMenu()
                    Box(modifier = Modifier.fillMaxWidth()) {
                        GemListRowView(
                            row = row,
                            listPosition = position,
                            modifier = Modifier.combinedClickable(
                                onClick = {},
                                onLongClick = { isShowDevelopEnable = true }.takeIf { opensMenu },
                            ),
                        )
                        if (opensMenu) {
                            DropdownMenu(
                                isShowDevelopEnable,
                                { isShowDevelopEnable = false },
                                containerColor = MaterialTheme.colorScheme.background,
                            ) {
                                DropdownMenuItem(
                                    text = { Text(viewState.developerToggle.string(context)) },
                                    onClick = {
                                        isShowDevelopEnable = false
                                        viewModel.toggleDeveloperMode()
                                    },
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
