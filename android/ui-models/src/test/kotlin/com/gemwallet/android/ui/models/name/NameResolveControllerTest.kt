package com.gemwallet.android.ui.models.name

import com.gemwallet.android.cases.name.ResolveName
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameProvider
import com.wallet.core.primitives.NameRecord
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class NameResolveControllerTest {

    private val chain = Chain.Ethereum

    private fun record(name: String = "vitalik.eth", address: String = "0xd8dA6B") = NameRecord(
        name = name,
        chain = chain,
        address = address,
        provider = NameProvider.Ens,
    )

    private class FakeResolveName(private val result: NameRecord?) : ResolveName {
        val requests = mutableListOf<Pair<String, Chain>>()

        override suspend fun resolveName(name: String, chain: Chain): NameRecord? {
            requests.add(name to chain)
            return result
        }
    }

    @Test
    fun plainAddressNeverReachesTheResolver() = runTest {
        val resolveName = FakeResolveName(record())
        val controller = NameResolveController(resolveName, this)

        controller.resolve("0xd8dA6B", chain)
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), resolveName.requests)
        assertEquals(NameRecordState.None, controller.state.value)
    }

    @Test
    fun rapidTypingResolvesOnlyTheLastValue() = runTest {
        val resolveName = FakeResolveName(record())
        val controller = NameResolveController(resolveName, this)

        controller.resolve("vit.eth", chain)
        controller.resolve("vita.eth", chain)
        controller.resolve("vitalik.eth", chain)
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), resolveName.requests)
        assertEquals(NameRecordState.Complete(record()), controller.state.value)
    }

    @Test
    fun missingAddressIsReportedAsError() = runTest {
        val resolveName = FakeResolveName(record(address = ""))
        val controller = NameResolveController(resolveName, this)

        controller.resolve("vitalik.eth", chain)
        advanceUntilIdle()

        assertEquals(NameRecordState.Error, controller.state.value)
    }

    @Test
    fun resetCancelsPendingResolve() = runTest {
        val resolveName = FakeResolveName(record())
        val controller = NameResolveController(resolveName, this)

        controller.resolve("vitalik.eth", chain)
        controller.reset()
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), resolveName.requests)
        assertEquals(NameRecordState.None, controller.state.value)
    }

    @Test
    fun onNameRecordDoesNotReResolveTheResolvedName() = runTest {
        val resolveName = FakeResolveName(record())
        val controller = NameResolveController(resolveName, this)

        controller.resolve("vitalik.eth", chain)
        advanceUntilIdle()
        controller.resolve("vitalik.eth", chain)
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), resolveName.requests)
        assertEquals(NameRecordState.Complete(record()), controller.state.value)
    }

    @Test
    fun emptyInputResetsResolvedState() = runTest {
        val resolveName = FakeResolveName(record())
        val controller = NameResolveController(resolveName, this)

        controller.resolve("vitalik.eth", chain)
        advanceUntilIdle()
        controller.resolve("", chain)
        advanceUntilIdle()

        assertEquals(NameRecordState.None, controller.state.value)
    }
}
