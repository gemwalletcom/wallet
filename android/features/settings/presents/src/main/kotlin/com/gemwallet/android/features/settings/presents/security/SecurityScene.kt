package com.gemwallet.android.features.settings.presents.security

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.features.settings.viewmodels.security.models.LockPeriodOption
import com.gemwallet.android.features.settings.viewmodels.security.models.SecurityRowAction
import com.gemwallet.android.features.settings.viewmodels.security.models.securityAction
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.OptionPickerRow
import com.gemwallet.android.ui.components.list_item.gemListSectionFooter
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.requestAuth
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListSection

@Composable
fun SecurityScene(
    sections: List<GemListSection>,
    lockInterval: Int?,
    lockPeriods: List<LockPeriodOption>,
    error: String?,
    isUpdatingAuthentication: Boolean,
    onErrorShown: () -> Unit,
    onAuthRequired: (Boolean) -> Unit,
    onHideBalances: () -> Unit,
    onLockInterval: (Int) -> Unit,
    onCancel: () -> Unit,
) {
    val context = LocalContext.current

    val snackbar = remember { SnackbarHostState() }
    LaunchedEffect(error) {
        error?.let {
            snackbar.showSnackbar(it)
            onErrorShown()
        }
    }

    Scene(
        title = stringResource(id = (R.string.settings_security)),
        onClose = onCancel,
        snackbar = snackbar,
    ) {
        LazyColumn {
            sections.forEachIndexed { index, section ->
                itemsPositioned(section.rows) { position, row ->
                    if (row is GemListRow.Picker) {
                        OptionPickerRow(
                            row = row,
                            listPosition = position,
                            current = lockPeriods.firstOrNull { it.minutes == lockInterval },
                            options = lockPeriods,
                            label = { stringResource(it.title) },
                            onSelect = { onLockInterval(it.minutes) },
                        )
                    } else {
                        GemListRowView(
                            row = row,
                            listPosition = position,
                            onToggle = { action, isOn ->
                                when (action.securityAction()) {
                                    SecurityRowAction.Authentication -> if (!isUpdatingAuthentication) {
                                        context.requestAuth(AuthRequest.Required) { onAuthRequired(isOn) }
                                    }

                                    SecurityRowAction.HideBalance -> onHideBalances()

                                    null -> Unit
                                }
                            },
                        )
                    }
                }
                gemListSectionFooter(section, key = "footer:$index")
            }
        }
    }
}
