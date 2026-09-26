package com.gemwallet.android.features.support.presents

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts.PickMultipleVisualMedia
import androidx.activity.result.contract.ActivityResultContracts.PickVisualMedia
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.support.viewmodels.SupportChatSceneViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyStateView
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.components.screen.rememberSnackbarState
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
fun SupportChatScreen(message: RouteMessage?, onMessageShown: () -> Unit, onCancel: () -> Unit, viewModel: SupportChatSceneViewModel = hiltViewModel()) {
    val days by viewModel.days.collectAsStateWithLifecycle()
    val isEmpty by viewModel.isEmpty.collectAsStateWithLifecycle()
    val typingAgentName by viewModel.typingAgentName.collectAsStateWithLifecycle()
    val error by viewModel.error.collectAsStateWithLifecycle()
    val errorRow by viewModel.errorRow.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = error, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)
    val context = LocalContext.current
    LaunchedEffect(message) {
        message?.let {
            snackbar.showSnackbar(it, context)
            onMessageShown()
        }
    }
    var previewUrl by remember { mutableStateOf<String?>(null) }

    val imagePicker = rememberLauncherForActivityResult(PickMultipleVisualMedia()) { uris ->
        viewModel.sendImages(uris)
    }

    LifecycleEventEffect(Lifecycle.Event.ON_RESUME) {
        viewModel.fetch()
    }

    Scene(
        titleContent = {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(paddingSmall),
            ) {
                Image(
                    painter = painterResource(R.drawable.support_agent),
                    contentDescription = null,
                    modifier = Modifier.size(compactIconSize).clip(CircleShape),
                )
                Text(
                    text = stringResource(R.string.settings_support),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        },
        onClose = onCancel,
        snackbar = snackbar,
    ) {
        Column(modifier = Modifier.fillMaxSize()) {
            Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
                SupportMessagesList(
                    days = days,
                    typingAgentName = typingAgentName,
                    onImageClick = { previewUrl = it },
                    onRetry = viewModel::retry,
                )
                when (val row = errorRow) {
                    null -> if (isEmpty) {
                        EmptyStateView(
                            title = stringResource(R.string.support_state_empty_title),
                            description = stringResource(R.string.support_state_empty_description),
                            iconVector = AppIcons.Article,
                            modifier = Modifier.align(Alignment.Center).padding(paddingDefault),
                        )
                    }

                    else -> GemListRowView(
                        row = row,
                        listPosition = ListPosition.Single,
                        modifier = Modifier.align(Alignment.Center).padding(paddingDefault),
                    )
                }
            }
            SupportInputBar(
                onPickImage = { imagePicker.launch(PickVisualMediaRequest(PickVisualMedia.ImageOnly)) },
                onSend = viewModel::sendText,
            )
        }
    }

    previewUrl?.let { url ->
        SupportImagePreviewDialog(url = url, onDismiss = { previewUrl = null })
    }
}
