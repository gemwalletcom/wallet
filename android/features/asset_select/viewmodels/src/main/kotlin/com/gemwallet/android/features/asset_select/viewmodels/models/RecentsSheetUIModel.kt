package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.model.RecentAsset
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.persistentListOf

data class RecentsSheetUIModel(
    val items: ImmutableList<RecentAsset>,
    val hasAnyRecents: Boolean,
    val searchActive: Boolean,
) {
    val isEmpty: Boolean get() = items.isEmpty()
    val showClear: Boolean get() = hasAnyRecents && !searchActive
    val emptyState: RecentsEmptyState? get() = when {
        !isEmpty -> null
        searchActive -> RecentsEmptyState.NoSearchResults
        else -> RecentsEmptyState.NoRecents
    }

    companion object {
        val Empty = RecentsSheetUIModel(
            items = persistentListOf(),
            hasAnyRecents = false,
            searchActive = false,
        )
    }
}

enum class RecentsEmptyState { NoRecents, NoSearchResults }
