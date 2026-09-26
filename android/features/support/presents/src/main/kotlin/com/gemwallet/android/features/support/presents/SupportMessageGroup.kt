package com.gemwallet.android.features.support.presents

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.gemwallet.android.features.support.viewmodels.SupportChatGroup
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageSender
import uniffi.gemstone.GemSupportMessageOutcome

@Composable
internal fun SupportMessageGroup(group: SupportChatGroup, onImageClick: (String) -> Unit, onRetry: (SupportMessage) -> Unit) {
    when (group.sender) {
        SupportMessageSender.User -> Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.End,
            verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
        ) {
            group.messages.forEach { item ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(paddingSmall),
                ) {
                    if (item.outcome is GemSupportMessageOutcome.Failed) {
                        Icon(
                            imageVector = AppIcons.Warning,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.error,
                        )
                    }
                    SupportMessageBubble(item = item, onImageClick = onImageClick, onRetry = onRetry)
                }
            }
        }

        is SupportMessageSender.Agent -> Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.Start,
            verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
        ) {
            group.messages.forEach { item ->
                SupportMessageBubble(item = item, onImageClick = onImageClick, onRetry = onRetry)
            }
        }
    }
}
