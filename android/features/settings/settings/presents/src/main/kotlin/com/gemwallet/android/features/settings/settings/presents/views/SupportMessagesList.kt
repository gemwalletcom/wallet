package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.isImeVisible
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.features.settings.settings.viewmodels.SupportChatDay
import com.gemwallet.android.features.settings.settings.viewmodels.SupportChatGroup
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.SupportMessage
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle

private sealed interface ChatRow {
    val key: String

    data class Separator(val day: SupportChatDay) : ChatRow {
        override val key: String = "separator:${day.date}"
    }

    data class Group(val group: SupportChatGroup) : ChatRow {
        override val key: String = "group:${group.messages.first().id}"
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun SupportMessagesList(
    days: List<SupportChatDay>,
    typingAgentName: String?,
    onImageClick: (String) -> Unit,
    onRetry: (SupportMessage) -> Unit,
) {
    val rows = remember(days) {
        buildList {
            days.forEach { day ->
                add(ChatRow.Separator(day))
                day.groups.forEach { add(ChatRow.Group(it)) }
            }
        }.asReversed()
    }
    val listState = rememberLazyListState()
    val newestMessageId = remember(days) {
        days.lastOrNull()?.groups?.lastOrNull()?.messages?.lastOrNull()?.id
    }
    LaunchedEffect(newestMessageId) {
        if (rows.isNotEmpty()) {
            listState.animateScrollToItem(0)
        }
    }

    val imeVisible = WindowInsets.isImeVisible
    LaunchedEffect(imeVisible) {
        if (imeVisible && rows.isNotEmpty()) {
            listState.animateScrollToItem(0)
        }
    }

    LaunchedEffect(typingAgentName) {
        if (typingAgentName != null) {
            listState.animateScrollToItem(0)
        }
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        state = listState,
        reverseLayout = true,
        contentPadding = PaddingValues(horizontal = paddingDefault, vertical = paddingSmall),
        verticalArrangement = Arrangement.spacedBy(paddingSmall, Alignment.Bottom),
    ) {
        val typingName = typingAgentName
        if (typingName != null) {
            item(key = "typing") {
                SupportTypingIndicator(name = typingName)
            }
        }
        items(rows, key = { it.key }, contentType = { it::class }) { row ->
            when (row) {
                is ChatRow.Separator -> DaySeparator(row.day)
                is ChatRow.Group -> SupportMessageGroup(group = row.group, onImageClick = onImageClick, onRetry = onRetry)
            }
        }
    }
}

@Composable
private fun DaySeparator(day: SupportChatDay) {
    Text(
        text = day.date.format(DateTimeFormatter.ofLocalizedDate(FormatStyle.MEDIUM)),
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.secondary,
        textAlign = TextAlign.Center,
        modifier = Modifier.fillMaxWidth().padding(vertical = paddingSmall),
    )
}
