package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.requiredSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import coil3.compose.SubcomposeAsyncImage
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.list_item.DropDownContextItem
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
internal val imageLoaderSize = 40.dp

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
    val time = DateFormat.getTimeInstance(DateFormat.SHORT).format(Date(message.createdAt))

    Column(
        modifier = Modifier.widthIn(max = messageBubbleMaxWidth),
        horizontalAlignment = if (isUser) Alignment.End else Alignment.Start,
        verticalArrangement = Arrangement.spacedBy(paddingHalfSmall),
    ) {
        message.images.forEach { image ->
            MessageImage(
                image = image,
                time = time,
                sending = message.status == SupportMessageStatus.Sending,
                onClick = onImageClick,
            )
        }
        if (message.content.isNotBlank()) {
            val context = LocalContext.current
            val clipboard = LocalClipboard.current.nativeClipboard
            var menuExpanded by remember { mutableStateOf(false) }
            val markdown = parseMarkdownToAnnotatedString(
                message.content,
                linkColor = if (isUser) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.primary,
            )
            val timeSpacerStyle = MaterialTheme.typography.labelSmall.toSpanStyle().copy(color = Color.Transparent)
            val text = buildAnnotatedString {
                append(markdown)
                withStyle(timeSpacerStyle) { append("  $time") }
            }
            DropDownContextItem(
                isExpanded = menuExpanded,
                onDismiss = { menuExpanded = false },
                onLongClick = { menuExpanded = true },
                onClick = {},
                menuItems = {
                    DropdownMenuItem(
                        text = { Text(stringResource(R.string.common_copy)) },
                        leadingIcon = { Icon(AppIcons.ContentCopy, contentDescription = null) },
                        onClick = {
                            clipboard.setPlainText(context, message.content)
                            menuExpanded = false
                        },
                    )
                },
                content = { contentModifier ->
                    Surface(
                        color = bubbleColor,
                        shape = RoundedCornerShape(messageBubbleCornerRadius),
                        modifier = contentModifier,
                    ) {
                        Box(modifier = Modifier.padding(horizontal = space12, vertical = paddingSmall)) {
                            Text(
                                text = text,
                                color = textColor,
                                style = MaterialTheme.typography.bodyLarge,
                            )
                            MessageMeta(
                                message = message,
                                time = time,
                                color = metaColor,
                                onRetry = onRetry,
                                modifier = Modifier.align(Alignment.BottomEnd),
                            )
                        }
                    }
                },
            )
        }
    }
}

@Composable
private fun MessageMeta(
    message: SupportMessage,
    time: String,
    color: Color,
    onRetry: (SupportMessage) -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(modifier = modifier, contentAlignment = Alignment.Center) {
        Text(
            text = time,
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
                    tint = color,
                    modifier = Modifier.size(statusIconSize).clickable { onRetry(message) },
                )
            } else {
                Icon(
                    imageVector = AppIcons.Error,
                    contentDescription = null,
                    tint = color,
                    modifier = Modifier.size(statusIconSize),
                )
            }
            SupportMessageStatus.Sent -> Unit
        }
    }
}

@Composable
private fun MessageImage(image: SupportMessageImage, time: String, sending: Boolean, onClick: (String) -> Unit) {
    Box(
        modifier = Modifier
            .size(width = attachmentImageWidth, height = attachmentImageHeight)
            .clip(RoundedCornerShape(space12))
            .background(MaterialTheme.colorScheme.surfaceContainerHighest)
            .clickable(enabled = image.url.isNotBlank()) { onClick(image.url) },
        contentAlignment = Alignment.Center,
    ) {
        if (image.url.isNotBlank()) {
            SubcomposeAsyncImage(
                model = image.url,
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize(),
                loading = { CircularProgressIndicator(modifier = Modifier.requiredSize(imageLoaderSize)) },
            )
        } else {
            CircularProgressIndicator(modifier = Modifier.requiredSize(imageLoaderSize))
        }
        if (!sending) {
            TimePill(time = time, modifier = Modifier.align(Alignment.BottomEnd))
        }
    }
}

@Composable
private fun TimePill(time: String, modifier: Modifier = Modifier) {
    Text(
        text = time,
        style = MaterialTheme.typography.labelSmall,
        color = Color.White,
        modifier = modifier
            .padding(paddingSmall)
            .background(Color.Black.copy(alpha = 0.5f), CircleShape)
            .padding(horizontal = paddingSmall, vertical = paddingHalfSmall),
    )
}
