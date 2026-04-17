package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.viewmodels.models.RecentsEmptyState
import com.gemwallet.android.features.asset_select.viewmodels.models.RecentsSheetUIModel
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.model.RecentAsset
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.wallet.core.primitives.AssetId
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale

@Composable
fun RecentsBottomSheet(
    isVisible: Boolean,
    uiModel: RecentsSheetUIModel,
    query: TextFieldState,
    onDismissRequest: () -> Unit,
    onClear: () -> Unit,
    onSelect: (AssetId) -> Unit,
) {
    val todayLabel = stringResource(R.string.date_today)
    val yesterdayLabel = stringResource(R.string.date_yesterday)
    val locale = LocalConfiguration.current.locales[0]

    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismissRequest,
        skipPartiallyExpanded = true,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight(0.95f),
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = paddingDefault),
                contentAlignment = Alignment.Center,
            ) {
                IconButton(
                    onClick = onDismissRequest,
                    modifier = Modifier.align(Alignment.CenterStart),
                ) {
                    Icon(
                        imageVector = Icons.Default.Close,
                        contentDescription = null,
                    )
                }
                Text(
                    text = stringResource(R.string.recent_activity_title),
                    style = MaterialTheme.typography.titleLarge,
                )
                if (uiModel.showClear) {
                    TextButton(
                        onClick = onClear,
                        modifier = Modifier.align(Alignment.CenterEnd),
                    ) {
                        Text(stringResource(R.string.filter_clear))
                    }
                }
            }
            SearchBar(query = query)
            val empty = uiModel.emptyState
            if (empty != null) {
                RecentsEmptyStateView(empty)
            } else {
                val sections = remember(uiModel.items, locale, todayLabel, yesterdayLabel) {
                    buildDateSections(uiModel.items, locale, todayLabel, yesterdayLabel)
                }
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    sections.forEach { section ->
                        recentsSection(section, onSelect)
                    }
                }
            }
        }
    }
}

private fun LazyListScope.recentsSection(
    section: RecentsDateSection,
    onSelect: (AssetId) -> Unit,
) {
    item(key = "header-${section.id}") {
        SubheaderItem(section.title)
    }
    itemsPositioned(
        section.items,
        key = { _, recent -> "${section.id}-${recent.asset.id.toIdentifier()}" },
    ) { position, recent ->
        AssetListItem(
            asset = recent.asset,
            listPosition = position,
            modifier = Modifier.clickable { onSelect(recent.asset.id) },
        )
    }
}

@Composable
private fun RecentsEmptyStateView(state: RecentsEmptyState) {
    val type = when (state) {
        RecentsEmptyState.NoRecents -> EmptyContentType.Recents
        RecentsEmptyState.NoSearchResults -> EmptyContentType.SearchAssets()
    }
    EmptyContentView(type = type, modifier = Modifier.fillMaxSize())
}

private data class RecentsDateSection(
    val id: String,
    val title: String,
    val items: List<RecentAsset>,
)

private fun buildDateSections(
    items: List<RecentAsset>,
    locale: Locale,
    todayLabel: String,
    yesterdayLabel: String,
): List<RecentsDateSection> {
    if (items.isEmpty()) return emptyList()
    val zone = ZoneId.systemDefault()
    val today = LocalDate.now(zone)
    val yesterday = today.minusDays(1)
    val longFormatter = DateTimeFormatter
        .ofLocalizedDate(FormatStyle.LONG)
        .withLocale(locale)

    return items.groupBy { recent ->
        Instant.ofEpochMilli(recent.addedAt).atZone(zone).toLocalDate()
    }
        .entries
        .sortedByDescending { it.key }
        .map { (date, values) ->
            val title = when (date) {
                today -> todayLabel
                yesterday -> yesterdayLabel
                else -> longFormatter.format(date)
            }
            RecentsDateSection(
                id = date.toString(),
                title = title,
                items = values.sortedByDescending { it.addedAt },
            )
        }
}
