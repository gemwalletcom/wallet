package com.gemwallet.android.features.asset_select.presents.views

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.viewmodels.models.RecentsEmptyState
import com.gemwallet.android.features.asset_select.viewmodels.models.RecentsSheetUIModel
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.dateGroupedList
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.AssetId

@Composable
fun RecentsBottomSheet(
    isVisible: Boolean,
    uiModel: RecentsSheetUIModel,
    query: TextFieldState,
    onDismissRequest: () -> Unit,
    onClear: () -> Unit,
    onSelect: (AssetId) -> Unit,
) {
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
                        imageVector = AppIcons.Close,
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
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    dateGroupedList(
                        items = uiModel.items.sortedByDescending { it.addedAt },
                        createdAt = { it.addedAt },
                        key = { _, recent -> "${recent.addedAt}-${recent.asset.id.toIdentifier()}" },
                    ) { position, recent ->
                        AssetListItem(
                            asset = recent.asset,
                            listPosition = position,
                            modifier = Modifier.clickable { onSelect(recent.asset.id) },
                        )
                    }
                }
            }
        }
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
