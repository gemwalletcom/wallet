package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.features.settings.settings.viewmodels.SupportChatGroup
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageStatus

@Composable
internal fun SupportMessageGroup(
    group: SupportChatGroup,
    onImageClick: (String) -> Unit,
    onRetry: (SupportMessage) -> Unit,
) {
    when (group) {
        is SupportChatGroup.User -> Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.End,
            verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
        ) {
            group.messages.forEach { message ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(paddingSmall),
                ) {
                    if (message.status == SupportMessageStatus.Failed) {
                        Icon(
                            imageVector = AppIcons.Warning,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.error,
                        )
                    }
                    SupportMessageBubble(message = message, onImageClick = onImageClick, onRetry = onRetry)
                }
            }
        }
        is SupportChatGroup.Agent -> Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.Start,
            verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
        ) {
            Text(
                text = group.name,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.secondary,
            )
            group.messages.forEach { message ->
                SupportMessageBubble(message = message, onImageClick = onImageClick, onRetry = onRetry)
            }
        }
    }
}
