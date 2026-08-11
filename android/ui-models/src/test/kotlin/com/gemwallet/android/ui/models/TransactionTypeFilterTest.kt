package com.gemwallet.android.ui.models

import com.wallet.core.primitives.TransactionType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class TransactionTypeFilterTest {

    @Test
    fun everyTransactionTypeIsSelectableByItsOwnGroup() {
        for (type in TransactionType.entries) {
            assertTrue(type.name, type.filterType.types.contains(type))
        }
    }

    @Test
    fun groupsCoverAllTypesWithoutOverlap() {
        val grouped = TransactionTypeFilter.entries.flatMap { it.types }

        assertEquals(TransactionType.entries.toSet(), grouped.toSet())
        assertEquals(TransactionType.entries.size, grouped.size)
    }
}
