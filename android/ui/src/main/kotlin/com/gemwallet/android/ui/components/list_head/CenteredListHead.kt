package com.gemwallet.android.ui.components.list_head

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.theme.headerIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall

enum class CenteredListHeadSubtitleLayout {
    Horizontal,
    Vertical,
}

@Composable
fun CenteredListHead(
    title: String,
    subtitle: String? = null,
    modifier: Modifier = Modifier,
    leading: (@Composable () -> Unit)? = null,
    subtitleLayout: CenteredListHeadSubtitleLayout = CenteredListHeadSubtitleLayout.Horizontal,
    bottomPadding: Dp = paddingDefault,
) {
    val subtitleText = subtitle?.takeIf { it.isNotBlank() }

    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(start = paddingDefault, end = paddingDefault, bottom = bottomPadding),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        leading?.let {
            it()
            Spacer(modifier = Modifier.height(paddingDefault))
        }
        when {
            subtitleText == null -> CenteredListHeadTitle(title)
            subtitleLayout == CenteredListHeadSubtitleLayout.Horizontal -> Row(
                horizontalArrangement = Arrangement.spacedBy(paddingHalfSmall),
                verticalAlignment = Alignment.Bottom,
            ) {
                CenteredListHeadTitle(title)
                CenteredListHeadSubtitle(subtitleText)
            }
            else -> {
                CenteredListHeadTitle(title)
                Spacer(modifier = Modifier.height(paddingHalfSmall))
                CenteredListHeadSubtitle(subtitleText)
            }
        }
    }
}

@Composable
fun CenteredListHead(
    icon: Any?,
    title: String,
    subtitle: String? = null,
    modifier: Modifier = Modifier,
    iconSize: Dp = headerIconSize,
    placeholderText: String? = title.firstOrNull()?.uppercaseChar()?.toString(),
    contentDescription: String = "centered_list_head_icon",
    subtitleLayout: CenteredListHeadSubtitleLayout = CenteredListHeadSubtitleLayout.Horizontal,
    bottomPadding: Dp = paddingDefault,
) {
    CenteredListHead(
        title = title,
        subtitle = subtitle,
        modifier = modifier,
        subtitleLayout = subtitleLayout,
        bottomPadding = bottomPadding,
        leading = {
            if (icon != null) {
                AsyncImage(
                    model = icon,
                    size = iconSize,
                    placeholderText = placeholderText,
                    contentDescription = contentDescription,
                )
            }
        },
    )
}

@Composable
private fun CenteredListHeadTitle(title: String) {
    Text(
        text = title,
        style = MaterialTheme.typography.titleMedium,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        textAlign = TextAlign.Center,
    )
}

@Composable
private fun CenteredListHeadSubtitle(subtitle: String) {
    Text(
        text = subtitle,
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.secondary,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        textAlign = TextAlign.Center,
    )
}
