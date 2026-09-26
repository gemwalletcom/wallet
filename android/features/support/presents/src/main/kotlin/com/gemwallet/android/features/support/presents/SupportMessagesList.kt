package com.gemwallet.android.features.support.presents

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
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
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.features.support.viewmodels.SupportChatDay
import com.gemwallet.android.features.support.viewmodels.SupportChatGroup
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.isKeyboardVisible
import com.gemwallet.android.ui.format.SectionDateFormatter
import com.gemwallet.android.ui.format.gemDay
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.SupportMessage
import java.time.LocalDate

private sealed interface ChatRow {
    val key: String

    data class Separator(val day: SupportChatDay) : ChatRow {
        override val key: String = "separator:${day.date}"
    }

    data class Group(val group: SupportChatGroup) : ChatRow {
        override val key: String = "group:${group.messages.first().id}"
    }
}

@Composable
internal fun SupportMessagesList(days: List<SupportChatDay>, typingAgentName: String?, onImageClick: (String) -> Unit, onRetry: (SupportMessage) -> Unit) {
    val todayLabel = stringResource(R.string.date_today)
    val yesterdayLabel = stringResource(R.string.date_yesterday)
    val boundaries = LocalDate.now().gemDay().boundaries()
    val dateFormatter = remember(todayLabel, yesterdayLabel, boundaries) {
        SectionDateFormatter(todayLabel, yesterdayLabel, boundaries)
    }
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

    val imeVisible = WindowInsets.isKeyboardVisible
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
        if (typingAgentName != null) {
            item(key = "typing") {
                SupportTypingIndicator()
            }
        }
        items(rows, key = { it.key }, contentType = { it::class }) { row ->
            when (row) {
                is ChatRow.Separator -> DaySeparator(row.day, dateFormatter)
                is ChatRow.Group -> SupportMessageGroup(group = row.group, onImageClick = onImageClick, onRetry = onRetry)
            }
        }
    }
}

@Composable
private fun DaySeparator(day: SupportChatDay, formatter: SectionDateFormatter) {
    Text(
        text = formatter.format(day.date, LocalConfiguration.current.locales[0]),
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.secondary,
        textAlign = TextAlign.Center,
        modifier = Modifier.fillMaxWidth().padding(vertical = paddingSmall),
    )
}
