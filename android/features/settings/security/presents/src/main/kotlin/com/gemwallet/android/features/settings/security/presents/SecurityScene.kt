package com.gemwallet.android.features.settings.security.presents

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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.security.viewmodels.SecurityViewModel
import com.gemwallet.android.features.settings.security.viewmodels.models.SecurityRowUIModel
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.compactIconSize

@Composable
fun SecurityScene(
    onCancel: () -> Unit,
    viewModel: SecurityViewModel = hiltViewModel(),
) {
    val rows by viewModel.rows.collectAsStateWithLifecycle()

    Scene(
        title = stringResource(id = (R.string.settings_security)),
        onClose = onCancel,
    ) {
        LazyColumn {
            rows.forEach { section ->
                itemsIndexed(section) { index, row ->
                    val listPosition = ListPosition.getPosition(index, section.size)
                    when (row) {
                        is SecurityRowUIModel.Authentication -> EnablePasscode(row, listPosition, viewModel::setAuthRequired)
                        is SecurityRowUIModel.LockPeriod -> RequiredAuthDelay(row, listPosition, viewModel::setLockInterval)
                        is SecurityRowUIModel.HideBalance -> HideBalanceItem(row, listPosition, viewModel::setHideBalances)
                    }
                }
            }
        }
    }
}

@Composable
private fun EnablePasscode(
    row: SecurityRowUIModel.Authentication,
    listPosition: ListPosition,
    onAuthRequired: (Boolean) -> Unit,
) {
    val context = LocalContext.current
    ListItem(
        model = row.model,
        listPosition = listPosition,
        accessory = {
            Switch(
                row.isEnabled,
                onCheckedChange = {
                    context.requestAuth(AuthRequest.Default) {
                        onAuthRequired(it)
                    }
                }
            )
        },
    )
}

@Composable
private fun RequiredAuthDelay(
    row: SecurityRowUIModel.LockPeriod,
    listPosition: ListPosition,
    onSelect: (Int) -> Unit,
) {
    var isShowLockDelays by remember { mutableStateOf(false) }
    ListItem(
        model = row.model,
        listPosition = listPosition,
        modifier = Modifier.clickable(onClick = { isShowLockDelays = true }),
        accessory = {
            DataBadgeChevron()
            DropdownMenu(
                expanded = isShowLockDelays,
                onDismissRequest = { isShowLockDelays = false },
                containerColor = MaterialTheme.colorScheme.background,
            ) {
                for (option in row.options) {
                    DropdownMenuItem(
                        text = {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                if (option.isSelected) {
                                    Icon(AppIcons.Check, null, modifier = Modifier.size(compactIconSize))
                                } else {
                                    Spacer(modifier = Modifier.size(compactIconSize))
                                }
                                Spacer4()
                                Text(stringResource(option.title))
                            }
                        },
                        {
                            onSelect(option.minutes)
                            isShowLockDelays = false
                        },
                    )
                }
            }
        },
    )
}

@Composable
private fun HideBalanceItem(
    row: SecurityRowUIModel.HideBalance,
    listPosition: ListPosition,
    onHide: () -> Unit,
) {
    ListItem(
        model = row.model,
        listPosition = listPosition,
        accessory = {
            Switch(
                checked = row.isEnabled,
                onCheckedChange = { onHide() }
            )
        },
    )
}
