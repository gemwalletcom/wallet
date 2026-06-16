package com.gemwallet.android.features.settings.settings.presents.views

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.settings.viewmodels.SupportChatSceneViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyStateView
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
fun SupportChatNavScreen(
    onCancel: () -> Unit,
    viewModel: SupportChatSceneViewModel = hiltViewModel(),
) {
    val days by viewModel.days.collectAsStateWithLifecycle()
    val isEmpty by viewModel.isEmpty.collectAsStateWithLifecycle()
    var previewUrl by remember { mutableStateOf<String?>(null) }

    val imagePicker = rememberLauncherForActivityResult(PickVisualMedia()) { uri ->
        uri?.let(viewModel::sendImage)
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
    ) {
        Column(modifier = Modifier.fillMaxSize()) {
            Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
                SupportMessagesList(
                    days = days,
                    onImageClick = { previewUrl = it },
                    onRetry = viewModel::retry,
                )
                if (isEmpty) {
                    EmptyStateView(
                        title = stringResource(R.string.support_state_empty_title),
                        description = stringResource(R.string.support_state_empty_description),
                        iconVector = AppIcons.Article,
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
