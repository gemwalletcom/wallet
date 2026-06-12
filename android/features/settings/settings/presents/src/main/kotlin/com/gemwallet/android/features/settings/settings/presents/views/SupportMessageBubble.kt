package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.gemwallet.android.ui.components.parseMarkdownToAnnotatedString
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space12
import com.wallet.core.primitives.SupportMessage
import com.wallet.core.primitives.SupportMessageImage
import com.wallet.core.primitives.SupportMessageSender
import com.wallet.core.primitives.SupportMessageStatus
import java.text.DateFormat
import java.util.Date

private val messageBubbleCornerRadius = 18.dp
private val messageBubbleMaxWidth = 300.dp
private val attachmentImageWidth = 240.dp
private val attachmentImageHeight = 180.dp
private val statusIconSize = 14.dp

@Composable
internal fun SupportMessageBubble(
    message: SupportMessage,
    onImageClick: (String) -> Unit,
    onRetry: (SupportMessage) -> Unit,
) {
    val isUser = message.sender is SupportMessageSender.User
    val bubbleColor = if (isUser) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceContainerHighest
    val textColor = if (isUser) Color.White else MaterialTheme.colorScheme.onSurface
    val metaColor = if (isUser) Color.White.copy(alpha = 0.7f) else MaterialTheme.colorScheme.secondary

    Column(
        modifier = Modifier.widthIn(max = messageBubbleMaxWidth),
        horizontalAlignment = if (isUser) Alignment.End else Alignment.Start,
        verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
    ) {
        message.images.forEach { image ->
            MessageImage(image = image, sending = message.status == SupportMessageStatus.Sending, onClick = onImageClick)
        }
        if (message.content.isNotBlank()) {
            Surface(color = bubbleColor, shape = RoundedCornerShape(messageBubbleCornerRadius)) {
                Row(
                    modifier = Modifier.padding(horizontal = space12, vertical = paddingSmall),
                    verticalAlignment = Alignment.Bottom,
                    horizontalArrangement = Arrangement.spacedBy(paddingSmall),
                ) {
                    Text(
                        text = parseMarkdownToAnnotatedString(message.content),
                        color = textColor,
                        style = MaterialTheme.typography.bodyLarge,
                        modifier = Modifier.weight(1f, fill = false),
                    )
                    MessageMeta(message = message, color = metaColor, onRetry = onRetry)
                }
            }
        } else {
            MessageMeta(message = message, color = MaterialTheme.colorScheme.secondary, onRetry = onRetry)
        }
    }
}

@Composable
private fun MessageMeta(
    message: SupportMessage,
    color: Color,
    onRetry: (SupportMessage) -> Unit,
) {
    Box(contentAlignment = Alignment.Center) {
        Text(
            text = DateFormat.getTimeInstance(DateFormat.SHORT).format(Date(message.createdAt)),
            style = MaterialTheme.typography.labelSmall,
            color = color,
            modifier = Modifier.alpha(if (message.status == SupportMessageStatus.Sent) 1f else 0f),
        )
        when (message.status) {
            SupportMessageStatus.Sending -> CircularProgressIndicator(
                modifier = Modifier.size(10.dp),
                strokeWidth = 1.5.dp,
                color = color,
            )
            SupportMessageStatus.Failed -> if (message.sender is SupportMessageSender.User && message.images.isEmpty()) {
                Icon(
                    imageVector = AppIcons.Refresh,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.error,
                    modifier = Modifier.size(statusIconSize).clickable { onRetry(message) },
                )
            } else {
                Icon(
                    imageVector = AppIcons.Error,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.error,
                    modifier = Modifier.size(statusIconSize),
                )
            }
            SupportMessageStatus.Sent -> Unit
        }
    }
}

@Composable
private fun MessageImage(image: SupportMessageImage, sending: Boolean, onClick: (String) -> Unit) {
    Box(
        modifier = Modifier
            .size(width = attachmentImageWidth, height = attachmentImageHeight)
            .clip(RoundedCornerShape(space12))
            .background(MaterialTheme.colorScheme.surfaceContainerHighest)
            .clickable(enabled = image.url.isNotBlank()) { onClick(image.url) },
        contentAlignment = Alignment.Center,
    ) {
        if (image.url.isNotBlank()) {
            AsyncImage(
                model = image.url,
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize(),
            )
        }
        if (sending) {
            CircularProgressIndicator()
        }
    }
}
