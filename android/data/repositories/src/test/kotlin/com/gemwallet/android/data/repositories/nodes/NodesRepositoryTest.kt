package com.gemwallet.android.data.repositories.nodes

import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.data.service.store.database.NodesDao
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.Config

class NodesRepositoryTest {

    private val nodesDao = mockk<NodesDao>(relaxed = true)
    private val configStore = mockk<ConfigStore>()
    private val config = mockk<Config>()

    @Test
    fun getCurrentBlockExplorer_fallsBackToSupportedExplorerWhenStoredValueIsInvalid() = runTest {
        every { configStore.getString("current_explorer", Chain.Near.string) } returns "NEAR Intents"
        every { config.getBlockExplorers(Chain.Near.string) } returns listOf("Near")
        every { config.getNodes() } returns emptyMap()

        val subject = NodesRepository(
            nodesDao = nodesDao,
            configStore = configStore,
            scope = backgroundScope,
            config = config,
        )

        assertEquals("Near", subject.getCurrentBlockExplorer(Chain.Near))
    }
}
