package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemNameInputStep
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemRecipientValidation

class NameServiceMock : GemNameServiceInterface {
    val requests = mutableListOf<Pair<String, Chain>>()

    override suspend fun getNameRecord(name: String, chain: uniffi.gemstone.Chain): GemNameRecordState {
        val requested = Chain.entries.first { it.string == chain }
        requests.add(name to requested)
        return GemNameRecordState.Complete(mockNameRecord(name = name, chain = requested).toGem())
    }

    override fun nameInputStep(state: GemNameRecordState, name: String, chain: uniffi.gemstone.Chain?): GemNameInputStep = when {
        name.isEmpty() || chain == null -> GemNameInputStep.Reset
        state is GemNameRecordState.Loading && state.name == name && state.chain == chain -> GemNameInputStep.Unchanged
        state is GemNameRecordState.Complete && state.record.name == name && state.record.chain == chain -> GemNameInputStep.Unchanged
        name.split(".").size < 2 -> GemNameInputStep.Reset
        else -> GemNameInputStep.Resolve(name, 500u)
    }

    override fun resolvedState(state: GemNameRecordState, name: String, chain: uniffi.gemstone.Chain, resolved: GemNameRecordState): GemNameRecordState =
        if (state is GemNameRecordState.Loading && state.name == name && state.chain == chain) resolved else state

    override fun validateRecipient(chain: uniffi.gemstone.Chain, input: String, state: GemNameRecordState): GemRecipientValidation = GemRecipientValidation(isValid = true, address = input, error = null)
}
