package com.gemwallet.android.features.asset_select.viewmodels.models

import com.gemwallet.android.model.RecentAsset
import kotlinx.collections.immutable.ImmutableList
import uniffi.gemstone.GemRecentsCounts
import uniffi.gemstone.GemRecentsSections
import kotlinx.collections.immutable.persistentListOf

data class RecentsSheetUIModel(
    val items: ImmutableList<RecentAsset>,
    val sections: GemRecentsSections,
) {
    val isEmpty: Boolean get() = !sections.showsItems
    val showClear: Boolean get() = sections.showsClear
    val emptyState: RecentsEmptyState? get() = when {
        !isEmpty -> null
        sections.showsNoResults -> RecentsEmptyState.NoSearchResults
        else -> RecentsEmptyState.NoRecents
    }

    companion object {
        val Empty = RecentsSheetUIModel(
            items = persistentListOf(),
            sections = GemRecentsCounts(recents = 0u, matching = 0u).sections(false),
        )
    }
}

enum class RecentsEmptyState { NoRecents, NoSearchResults }
