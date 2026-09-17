package com.gemwallet.android.ui.models.name

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.NameServiceMock
import com.gemwallet.android.testkit.mockNameRecord
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemNameRecordState

@OptIn(ExperimentalCoroutinesApi::class)
class NameRecordControllerTest {

    private val chain = Chain.Ethereum

    private val complete = GemNameRecordState.Complete(mockNameRecord().toGem())

    @Test
    fun plainAddressNeverReachesTheResolver() = runTest {
        val getNameRecord = NameServiceMock()
        val controller = NameRecordController(getNameRecord, this)

        controller.getNameRecord("0xd8dA6B", chain)
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), getNameRecord.requests)
        assertEquals(GemNameRecordState.None, controller.state.value)
    }

    @Test
    fun rapidTypingResolvesOnlyTheLastValue() = runTest {
        val getNameRecord = NameServiceMock()
        val controller = NameRecordController(getNameRecord, this)

        controller.getNameRecord("vit.eth", chain)
        controller.getNameRecord("vita.eth", chain)
        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), getNameRecord.requests)
        assertEquals(complete, controller.state.value)
    }

    @Test
    fun resetCancelsPendingResolve() = runTest {
        val getNameRecord = NameServiceMock()
        val controller = NameRecordController(getNameRecord, this)

        controller.getNameRecord("vitalik.eth", chain)
        controller.reset()
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), getNameRecord.requests)
        assertEquals(GemNameRecordState.None, controller.state.value)
    }

    @Test
    fun onNameRecordDoesNotReResolveTheResolvedName() = runTest {
        val getNameRecord = NameServiceMock()
        val controller = NameRecordController(getNameRecord, this)

        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()
        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), getNameRecord.requests)
        assertEquals(complete, controller.state.value)
    }

    @Test
    fun emptyInputResetsResolvedState() = runTest {
        val getNameRecord = NameServiceMock()
        val controller = NameRecordController(getNameRecord, this)

        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()
        controller.getNameRecord("", chain)
        advanceUntilIdle()

        assertEquals(GemNameRecordState.None, controller.state.value)
    }
}
