package com.gemwallet.android.features.update_app.presents

import android.content.Intent
import android.content.Intent.FLAG_ACTIVITY_NEW_TASK
import android.provider.Settings
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowCircleDown
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.core.net.toUri
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.update_app.viewmodels.DownloadState
import com.gemwallet.android.features.update_app.viewmodels.InAppUpdateViewModel
import com.gemwallet.android.model.AppUpdateInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.DropDownContextItem
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.mainActionHeight
import com.gemwallet.android.ui.theme.space0
import com.gemwallet.android.ui.theme.space2
import kotlin.math.roundToInt

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun InAppUpdateBanner() {
    val viewModel: InAppUpdateViewModel = hiltViewModel()

    val updateAvailable by viewModel.updateAvailable.collectAsStateWithLifecycle()
    val state by viewModel.downloadState.collectAsStateWithLifecycle()

    var isShowContextMenu by remember { mutableStateOf(false) }

    val update = updateAvailable ?: return
    if (state == DownloadState.Success) return
    val canDismiss = !update.isRequired

    val action = {
        when (state) {
            DownloadState.Error,
            DownloadState.Success,
            DownloadState.Canceled,
            DownloadState.PermissionRequired,
            DownloadState.Idle -> viewModel.update()

            DownloadState.Preparing,
            is DownloadState.Progress -> {
                if (canDismiss) viewModel.cancel()
            }
        }
    }

    DropDownContextItem(
        modifier = Modifier.listItem(ListPosition.Single),
        isExpanded = isShowContextMenu,
        onDismiss = { isShowContextMenu = false },
        content = { modifier ->
            UpdateInfo(
                modifier = modifier,
                state = state,
                updateAvailable = update,
                onAction = action,
            )
        },
        menuItems = {
            when (state) {
                DownloadState.Idle,
                DownloadState.Error,
                DownloadState.Canceled,
                DownloadState.PermissionRequired,
                DownloadState.Success -> {
                    DropdownMenuItem(
                        text = { Text(text = stringResource(id = R.string.update_app_action)) },
                        onClick = {
                            isShowContextMenu = false
                            viewModel.update()
                        },
                    )
                    if (canDismiss) {
                        DropdownMenuItem(
                            text = { Text(text = stringResource(R.string.common_skip)) },
                            onClick = {
                                isShowContextMenu = false
                                viewModel.skip()
                            },
                        )
                    }
                }
                DownloadState.Preparing,
                is DownloadState.Progress -> {
                    if (canDismiss) {
                        DropdownMenuItem(
                            text = { Text(text = stringResource(id = R.string.common_cancel)) },
                            onClick = {
                                isShowContextMenu = false
                                viewModel.cancel()
                            },
                        )
                    }
                }
            }
        },
        onLongClick = { isShowContextMenu = true },
        onClick = action,
    )
    RequestInstallPermissions(
        isVisible = state == DownloadState.PermissionRequired,
        onDismiss = viewModel::dismissPermissionPrompt,
    )
}

@Composable
private fun UpdateInfo(
    modifier: Modifier = Modifier,
    state: DownloadState,
    updateAvailable: AppUpdateInfo,
    onAction: () -> Unit,
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .defaultPadding(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(text = stringResource(R.string.update_app_title))
            Text(
                text = if (state is DownloadState.Error) {
                    "${updateAvailable.version} · ${stringResource(R.string.common_try_again)}"
                } else {
                    updateAvailable.version
                },
                style = MaterialTheme.typography.bodyMedium,
                color = if (state is DownloadState.Error) {
                    MaterialTheme.colorScheme.error
                } else {
                    MaterialTheme.colorScheme.secondary
                },
            )
        }
        Box(
            modifier = Modifier.height(mainActionHeight),
            contentAlignment = Alignment.Center,
        ) {
            when (state) {
                DownloadState.Error,
                DownloadState.Success,
                DownloadState.Canceled,
                DownloadState.PermissionRequired,
                DownloadState.Idle -> TextButton(
                    onClick = onAction,
                    contentPadding = PaddingValues(space0),
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(stringResource(R.string.update_app_action))
                        Spacer4()
                        Icon(Icons.Default.ArrowCircleDown, "Update application", tint = MaterialTheme.colorScheme.primary)
                    }
                }
                DownloadState.Preparing -> CircularProgressIndicator(
                    modifier = Modifier.size(iconSize),
                    strokeWidth = space2,
                )
                is DownloadState.Progress -> Box {
                    val fraction = state.fraction
                    if (fraction == null) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(iconSize),
                            strokeWidth = space2,
                        )
                    } else {
                        CircularProgressIndicator(
                            modifier = Modifier.size(iconSize),
                            strokeWidth = space2,
                            progress = { fraction },
                        )
                        Text(
                            modifier = Modifier.align(Alignment.Center),
                            text = "${(fraction * 100).roundToInt()}%",
                            style = MaterialTheme.typography.labelSmall,
                        )
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun RequestInstallPermissions(
    isVisible: Boolean,
    onDismiss: () -> Unit,
) {
    if (!isVisible) {
        return
    }
    val context = LocalContext.current
    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = MaterialTheme.colorScheme.background,
        confirmButton = {
            Button(
                onClick = {
                    onDismiss()
                    val intent = Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES).apply {
                        data = "package:${context.packageName}".toUri()
                        addFlags(FLAG_ACTIVITY_NEW_TASK)
                    }
                    context.startActivity(intent)
                }
            ) {
                Text(stringResource(R.string.update_app_permission_open_settings))
            }
        },
        dismissButton = {
            Button(onClick = onDismiss) {
                Text(stringResource(R.string.common_cancel))
            }
        },
        title = {
            Text(text = stringResource(R.string.update_app_permission_title))
        },
        text = {
            Text(text = stringResource(R.string.update_app_permission_description))
        }
    )
}
