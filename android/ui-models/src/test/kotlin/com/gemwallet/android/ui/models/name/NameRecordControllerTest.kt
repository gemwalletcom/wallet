package com.gemwallet.android.ui.models.name

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameProvider
import com.wallet.core.primitives.NameRecord
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemNameInputStep

@OptIn(ExperimentalCoroutinesApi::class)
class NameRecordControllerTest {

    private val chain = Chain.Ethereum

    private fun record() = NameRecord(
        name = "vitalik.eth",
        chain = chain,
        address = "0xd8dA6B",
        provider = NameProvider.Ens,
    )

    private fun complete() = GemNameRecordState.Complete(record().toGem())

    private class FakeGetNameRecord(private val result: GemNameRecordState) {
        val requests = mutableListOf<Pair<String, Chain>>()

        fun service(): GemNameServiceInterface = mockk(relaxed = true) {
            every { isNameSupported(any()) } answers { firstArg<String>().split(".").size >= 2 }
            every { nameInputStep(any(), any(), any()) } answers {
                val state = firstArg<GemNameRecordState>()
                val name = secondArg<String>()
                when {
                    name.isEmpty() -> GemNameInputStep.Reset
                    state is GemNameRecordState.Loading && state.name == name -> GemNameInputStep.Unchanged
                    state is GemNameRecordState.Complete && state.record.name == name -> GemNameInputStep.Unchanged
                    !thirdArg<Boolean>() || name.split(".").size < 2 -> GemNameInputStep.Reset
                    else -> GemNameInputStep.Resolve(name, 500u)
                }
            }
            every { resolvedState(any(), any(), any()) } answers {
                val state = firstArg<GemNameRecordState>()
                val name = secondArg<String>()
                if (state is GemNameRecordState.Loading && state.name == name) thirdArg() else state
            }
            coEvery { getNameRecord(any(), any()) } answers {
                requests.add(firstArg<String>() to Chain.entries.first { it.string == secondArg<String>() })
                result
            }
        }
    }

    @Test
    fun plainAddressNeverReachesTheResolver() = runTest {
        val getNameRecord = FakeGetNameRecord(complete())
        val controller = NameRecordController(getNameRecord.service(), this)

        controller.getNameRecord("0xd8dA6B", chain)
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), getNameRecord.requests)
        assertEquals(GemNameRecordState.None, controller.state.value)
    }

    @Test
    fun rapidTypingResolvesOnlyTheLastValue() = runTest {
        val getNameRecord = FakeGetNameRecord(complete())
        val controller = NameRecordController(getNameRecord.service(), this)

        controller.getNameRecord("vit.eth", chain)
        controller.getNameRecord("vita.eth", chain)
        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), getNameRecord.requests)
        assertEquals(complete(), controller.state.value)
    }

    @Test
    fun resetCancelsPendingResolve() = runTest {
        val getNameRecord = FakeGetNameRecord(complete())
        val controller = NameRecordController(getNameRecord.service(), this)

        controller.getNameRecord("vitalik.eth", chain)
        controller.reset()
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), getNameRecord.requests)
        assertEquals(GemNameRecordState.None, controller.state.value)
    }

    @Test
    fun onNameRecordDoesNotReResolveTheResolvedName() = runTest {
        val getNameRecord = FakeGetNameRecord(complete())
        val controller = NameRecordController(getNameRecord.service(), this)

        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()
        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), getNameRecord.requests)
        assertEquals(complete(), controller.state.value)
    }

    @Test
    fun emptyInputResetsResolvedState() = runTest {
        val getNameRecord = FakeGetNameRecord(complete())
        val controller = NameRecordController(getNameRecord.service(), this)

        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()
        controller.getNameRecord("", chain)
        advanceUntilIdle()

        assertEquals(GemNameRecordState.None, controller.state.value)
    }
}
