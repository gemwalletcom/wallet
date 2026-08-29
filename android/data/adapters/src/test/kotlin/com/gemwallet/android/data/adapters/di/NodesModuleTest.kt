package com.gemwallet.android.data.adapters.di

import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemNodeService

class NodesModuleTest {

    @Test
    fun getNodeUrlCaseUsesSelectedPreferenceWithoutLoadingNodes() {
        val service = mockk<GemNodeService>()
        val selectedUrl = "https://custom.example"
        every { service.nodeUrl(Chain.Ethereum.string) } returns selectedUrl

        val getNodeUrlCase = NodesModule.provideGetNodeUrlCase(service)

        assertEquals(selectedUrl, getNodeUrlCase.getNodeUrl(Chain.Ethereum))
        verify(exactly = 1) { service.nodeUrl(Chain.Ethereum.string) }
    }
}
