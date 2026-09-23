package com.gemwallet.android.features.settings.security.presents

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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.security.viewmodels.SecurityViewModel
import com.gemwallet.android.features.settings.security.viewmodels.models.SecurityRowAction
import com.gemwallet.android.features.settings.security.viewmodels.models.securityAction
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.compactIconSize

@Composable
fun SecurityScene(onCancel: () -> Unit, viewModel: SecurityViewModel = hiltViewModel()) {
    val context = LocalContext.current
    val sections by viewModel.sections.collectAsStateWithLifecycle()
    val lockInterval by viewModel.lockInterval.collectAsStateWithLifecycle(null)
    var isShowLockPeriods by remember { mutableStateOf(false) }

    Scene(
        title = stringResource(id = (R.string.settings_security)),
        onClose = onCancel,
    ) {
        LazyColumn {
            sections.forEach { section ->
                itemsPositioned(section.rows) { position, row ->
                    GemListRowView(
                        row = row,
                        listPosition = position,
                        onToggle = { title, isOn ->
                            when (title.securityAction()) {
                                SecurityRowAction.Authentication -> context.requestAuth(AuthRequest.Required) { viewModel.setAuthRequired(isOn) }
                                SecurityRowAction.HideBalance -> viewModel.setHideBalances()
                                null -> Unit
                            }
                        },
                        onSelect = { isShowLockPeriods = true },
                        accessory = {
                            DropdownMenu(
                                expanded = isShowLockPeriods,
                                onDismissRequest = { isShowLockPeriods = false },
                                containerColor = MaterialTheme.colorScheme.background,
                            ) {
                                for (option in viewModel.lockPeriods) {
                                    DropdownMenuItem(
                                        text = {
                                            Row(verticalAlignment = Alignment.CenterVertically) {
                                                if (option.minutes == lockInterval) {
                                                    Icon(AppIcons.Check, null, modifier = Modifier.size(compactIconSize))
                                                } else {
                                                    Spacer(modifier = Modifier.size(compactIconSize))
                                                }
                                                Spacer4()
                                                Text(stringResource(option.title))
                                            }
                                        },
                                        {
                                            viewModel.setLockInterval(option.minutes)
                                            isShowLockPeriods = false
                                        },
                                    )
                                }
                            }
                        },
                    )
                }
            }
        }
    }
}
