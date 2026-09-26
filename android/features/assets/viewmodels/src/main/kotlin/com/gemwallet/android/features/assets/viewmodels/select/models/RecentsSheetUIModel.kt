package com.gemwallet.android.features.assets.viewmodels.select.models

import com.gemwallet.android.model.RecentAsset
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.persistentListOf
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemRecentsCounts
import uniffi.gemstone.GemRecentsSections

data class RecentsSheetUIModel(val items: ImmutableList<RecentAsset>, val sections: GemRecentsSections) {
    val isEmpty: Boolean get() = !sections.showsItems
    val showClear: Boolean get() = sections.showsClear
    val emptyState: GemEmptyStateKind? get() = sections.empty

    companion object {
        val Empty = RecentsSheetUIModel(
            items = persistentListOf(),
            sections = GemRecentsCounts(recents = 0u, matching = 0u).sections(false),
        )
    }
}
