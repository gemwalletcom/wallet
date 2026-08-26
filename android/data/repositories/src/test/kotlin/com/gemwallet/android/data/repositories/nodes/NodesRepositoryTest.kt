package com.gemwallet.android.data.repositories.nodes

import com.gemwallet.android.data.service.store.ConfigStore
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.Config
import uniffi.gemstone.GemNodeService

class NodesRepositoryTest {

    private val configStore = mockk<ConfigStore>()
    private val config = mockk<Config>()

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun getCurrentBlockExplorer_fallsBackToSupportedExplorerWhenStoredValueIsInvalid() = runTest {
        every { configStore.getString("current_explorer", Chain.Near.string) } returns "NEAR Intents"
        every { config.getBlockExplorers(Chain.Near.string) } returns listOf("Near")

        val subject = NodesRepository(
            nodeService = mockk<GemNodeService>(),
            configStore = configStore,
            config = config,
        )

        assertEquals("Near", subject.getCurrentBlockExplorer(Chain.Near))
    }
}
