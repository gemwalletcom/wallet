package com.gemwallet.android.features.assets.presents.select

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
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.SearchBar
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.list_item.dateSectionedList
import com.gemwallet.android.ui.components.list_item.rememberDaySections
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.SheetSizing
import com.gemwallet.android.ui.theme.paddingDefault
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemRecentsViewState

@Composable
fun RecentsScene(isVisible: Boolean, viewState: GemRecentsViewState, query: TextFieldState, onDismissRequest: () -> Unit, onClear: () -> Unit, onSelect: (Asset) -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        onDismissRequest = onDismissRequest,
        expansion = SheetExpansion.Full,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight(SheetSizing.heightFraction),
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
                if (viewState.sections.showsClear) {
                    TextButton(
                        onClick = onClear,
                        modifier = Modifier.align(Alignment.CenterEnd),
                    ) {
                        Text(stringResource(R.string.filter_clear))
                    }
                }
            }
            SearchBar(query = query)
            val sections = rememberDaySections(viewState.days, day = { it.day }, items = { day -> day.recents.map { it.toPrimitives() } })
            val empty = viewState.sections.empty
            if (empty != null) {
                RecentsEmptyStateView(empty)
            } else {
                LazyColumn(modifier = Modifier.fillMaxSize()) {
                    dateSectionedList(
                        sections = sections,
                        key = { _, recent -> "${recent.createdAt}-${recent.asset.id.toIdentifier()}" },
                    ) { position, recent ->
                        AssetListItem(
                            asset = recent.asset,
                            listPosition = position,
                            modifier = Modifier.clickable { onSelect(recent.asset) },
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun RecentsEmptyStateView(kind: GemEmptyStateKind) {
    EmptyContentView(kind = kind, modifier = Modifier.fillMaxSize())
}
