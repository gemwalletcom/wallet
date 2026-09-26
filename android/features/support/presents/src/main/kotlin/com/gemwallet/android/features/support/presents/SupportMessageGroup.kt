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
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.SupportMessage
import uniffi.gemstone.GemSupportBubbleSide
import uniffi.gemstone.GemSupportChatGroup
import uniffi.gemstone.GemSupportMessageOutcome

@Composable
internal fun SupportMessageGroup(group: GemSupportChatGroup, onImageClick: (String) -> Unit, onRetry: (SupportMessage) -> Unit) {
    when (group.side) {
        GemSupportBubbleSide.OUTGOING -> Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.End,
            verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
        ) {
            group.rows.forEach { row ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(paddingSmall),
                ) {
                    if (row.outcome is GemSupportMessageOutcome.Failed) {
                        Icon(
                            imageVector = AppIcons.Warning,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.error,
                        )
                    }
                    SupportMessageBubble(row = row, onImageClick = onImageClick, onRetry = onRetry)
                }
            }
        }

        GemSupportBubbleSide.INCOMING -> Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.Start,
            verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
        ) {
            group.rows.forEach { row ->
                SupportMessageBubble(row = row, onImageClick = onImageClick, onRetry = onRetry)
            }
        }
    }
}
