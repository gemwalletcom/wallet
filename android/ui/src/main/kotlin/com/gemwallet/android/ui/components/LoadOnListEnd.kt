package com.gemwallet.android.ui.components

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.snapshotFlow
import com.gemwallet.android.ext.GemConstants
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull

@Composable
fun LoadOnListEnd(listState: LazyListState, onEndReached: () -> Unit) {
    val onEndReachedCurrent by rememberUpdatedState(onEndReached)

    LaunchedEffect(listState) {
        snapshotFlow {
            listState.layoutInfo.totalItemsCount.takeIf { count ->
                (listState.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: -1) >= count - GemConstants.listPrefetchRows
            }
        }
            .filterNotNull()
            .distinctUntilChanged()
            .collect { onEndReachedCurrent() }
    }
}
