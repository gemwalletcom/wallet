package com.gemwallet.android.features.wallets.presents.views

import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.gemwallet.android.testkit.mockGemWalletRow
import com.gemwallet.android.testkit.mockWalletDataAggregate
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class WalletsScreenTest {

    @Test
    fun `toWalletSections partitions wallets from one snapshot`() {
        val pinnedWallet = mockWalletDataAggregate(mockGemWalletRow(id = "pinned", isPinned = true))
        val unpinnedWallet = mockWalletDataAggregate(mockGemWalletRow(id = "unpinned"))
        val secondPinnedWallet = mockWalletDataAggregate(mockGemWalletRow(id = "second-pinned", isPinned = true))

        val sections = listOf(pinnedWallet, unpinnedWallet, secondPinnedWallet).toWalletSections()

        assertEquals(listOf(pinnedWallet, secondPinnedWallet), sections.pinnedWallets)
        assertEquals(listOf(unpinnedWallet), sections.unpinnedWallets)
        assertEquals(
            listOf("pinned", "second-pinned", "unpinned"),
            sections.allWallets.map { it.row.id }
        )
    }

    @Test
    fun `toWalletSections keeps empty section lists stable`() {
        val sections = emptyList<WalletDataAggregate>().toWalletSections()

        assertTrue(sections.pinnedWallets.isEmpty())
        assertTrue(sections.unpinnedWallets.isEmpty())
        assertTrue(sections.allWallets.isEmpty())
    }
}
