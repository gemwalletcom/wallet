package com.gemwallet.android.features.settings.security.presents

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
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
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.features.settings.security.presents.localization.stringRes
import uniffi.gemstone.GemLockPeriod
import uniffi.gemstone.lockPeriodFromMinutes
import uniffi.gemstone.lockPeriods
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyTitleText
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemSecurityRow
import com.gemwallet.android.ui.requestAuth
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.compactIconSize

@Composable
fun SecurityScene(
    onCancel: () -> Unit,
    viewModel: SecurityViewModel = hiltViewModel(),
) {
    var authRequired by remember { mutableStateOf(viewModel.authRequired()) }
    val hideBalances by viewModel.isHideBalances.collectAsStateWithLifecycle()
    val lockInterval by viewModel.lockInterval.collectAsStateWithLifecycle()
    val lockPeriods = remember { lockPeriods() }
    val sections = remember(authRequired) { viewModel.sections(authRequired) }
    val currentLockPeriod = remember(lockInterval) { lockPeriodFromMinutes(lockInterval.toUInt()) }

    Scene(
        title = stringResource(id = (R.string.settings_security)),
        onClose = onCancel,
    ) {
        LazyColumn {
            sections.forEach { section ->
                section.rows.forEach { row ->
                    when (row) {
                        GemSecurityRow.AUTHENTICATION -> enablePasscode(authRequired) {
                            viewModel.setAuthRequired(it)
                            authRequired = it
                        }
                        GemSecurityRow.LOCK_PERIOD -> requiredAuthDelay(lockPeriods, lockInterval, currentLockPeriod, viewModel::setLockInterval)
                        GemSecurityRow.PRIVACY_LOCK -> Unit
                        GemSecurityRow.HIDE_BALANCE -> hideBalanceItem(hideBalances, viewModel::setHideBalances)
                    }
                }
            }
        }
    }
}

private fun LazyListScope.enablePasscode(
    authRequired: Boolean,
    onAuthRequired: (Boolean) -> Unit,
) {
    item {
        val context = LocalContext.current
        PropertyItem(
            title = { PropertyTitleText(R.string.settings_enable_passcode) },
            data = {
                Switch(
                    authRequired,
                    onCheckedChange = {
                        context.requestAuth(AuthRequest.Default) {
                            onAuthRequired(it)
                        }
                    }
                )
            },
            listPosition = if (authRequired) ListPosition.First else ListPosition.Single,
        )
    }
}

private fun LazyListScope.requiredAuthDelay(
    locks: List<GemLockPeriod>,
    currentInterval: Int,
    currentPeriod: GemLockPeriod,
    onSelect: (Int) -> Unit,
) {
    item {
        var isShowLockDelays by remember { mutableStateOf(false) }
        PropertyItem(
            modifier = Modifier.clickable(onClick = { isShowLockDelays = true }),
            title = { PropertyTitleText(R.string.lock_require_authentication) },
            data = {
                PropertyDataText(text = stringResource(currentPeriod.stringRes()))
                DropdownMenu(
                    expanded = isShowLockDelays,
                    onDismissRequest = { isShowLockDelays = false },
                    containerColor = MaterialTheme.colorScheme.background,
                ) {
                    for (period in locks) {
                        val interval = period.minutes().toInt()
                        DropdownMenuItem(
                            text = {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    interval.takeIf { it == currentInterval }?.let {
                                        Icon(AppIcons.Check, null, modifier = Modifier.size(compactIconSize))
                                    } ?: Spacer(modifier = Modifier.size(compactIconSize))
                                    Spacer4()
                                    Text(stringResource(period.stringRes()))
                                }
                            },
                            {
                                onSelect(interval)
                                isShowLockDelays = false
                            },
                        )
                    }
                }
            },
            listPosition = ListPosition.Last,
        )
    }
}

private fun LazyListScope.hideBalanceItem(
    hideBalances: Boolean,
    onHide: () -> Unit,
) {
    item {
        PropertyItem(
            title = { PropertyTitleText(R.string.settings_hide_balance) },
            data = {
                Switch(
                    checked = hideBalances,
                    onCheckedChange = { onHide() }
                )
            },
            listPosition = ListPosition.Single,
        )
    }
}
