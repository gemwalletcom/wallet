package com.gemwallet.android.ui.models.name

import com.gemwallet.android.application.recipient.cases.GetNameRecord
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameProvider
import com.wallet.core.primitives.NameRecord
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class NameRecordControllerTest {

    private val chain = Chain.Ethereum

    private fun record(name: String = "vitalik.eth", address: String = "0xd8dA6B") = NameRecord(
        name = name,
        chain = chain,
        address = address,
        provider = NameProvider.Ens,
    )

    private class FakeGetNameRecord(private val result: NameRecord?) : GetNameRecord {
        val requests = mutableListOf<Pair<String, Chain>>()

        override suspend fun getNameRecord(name: String, chain: Chain): NameRecord? {
            requests.add(name to chain)
            return result
        }

        override fun isNameSupported(name: String): Boolean = name.split(".").size >= 2
    }

    @Test
    fun plainAddressNeverReachesTheResolver() = runTest {
        val getNameRecord = FakeGetNameRecord(record())
        val controller = NameRecordController(getNameRecord, this)

        controller.getNameRecord("0xd8dA6B", chain)
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), getNameRecord.requests)
        assertEquals(NameRecordState.None, controller.state.value)
    }

    @Test
    fun rapidTypingResolvesOnlyTheLastValue() = runTest {
        val getNameRecord = FakeGetNameRecord(record())
        val controller = NameRecordController(getNameRecord, this)

        controller.getNameRecord("vit.eth", chain)
        controller.getNameRecord("vita.eth", chain)
        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), getNameRecord.requests)
        assertEquals(NameRecordState.Complete(record()), controller.state.value)
    }

    @Test
    fun missingAddressIsReportedAsError() = runTest {
        val getNameRecord = FakeGetNameRecord(record(address = ""))
        val controller = NameRecordController(getNameRecord, this)

        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()

        assertEquals(NameRecordState.Error, controller.state.value)
    }

    @Test
    fun resetCancelsPendingResolve() = runTest {
        val getNameRecord = FakeGetNameRecord(record())
        val controller = NameRecordController(getNameRecord, this)

        controller.getNameRecord("vitalik.eth", chain)
        controller.reset()
        advanceUntilIdle()

        assertEquals(emptyList<Pair<String, Chain>>(), getNameRecord.requests)
        assertEquals(NameRecordState.None, controller.state.value)
    }

    @Test
    fun onNameRecordDoesNotReResolveTheResolvedName() = runTest {
        val getNameRecord = FakeGetNameRecord(record())
        val controller = NameRecordController(getNameRecord, this)

        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()
        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()

        assertEquals(listOf("vitalik.eth" to chain), getNameRecord.requests)
        assertEquals(NameRecordState.Complete(record()), controller.state.value)
    }

    @Test
    fun emptyInputResetsResolvedState() = runTest {
        val getNameRecord = FakeGetNameRecord(record())
        val controller = NameRecordController(getNameRecord, this)

        controller.getNameRecord("vitalik.eth", chain)
        advanceUntilIdle()
        controller.getNameRecord("", chain)
        advanceUntilIdle()

        assertEquals(NameRecordState.None, controller.state.value)
    }
}
