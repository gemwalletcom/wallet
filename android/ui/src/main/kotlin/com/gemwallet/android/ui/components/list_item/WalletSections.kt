package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.models.ListPosition
import uniffi.gemstone.GemWalletSectionKind

fun LazyListScope.walletSections(sections: List<WalletSectionUIModel>, selectedId: String?, onSelect: (String) -> Unit) {
    sections.forEach { section ->
        if (section.kind == GemWalletSectionKind.PINNED) {
            pinnedHeader()
        }
        itemsIndexed(section.rows, key = { _, row -> "${section.kind}-${row.id}" }) { index, row ->
            WalletItem(
                model = row,
                isCurrent = row.id == selectedId,
                listPosition = ListPosition.getPosition(index, section.rows.size),
                modifier = Modifier.clickable { onSelect(row.id) },
            )
        }
    }
}
