package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemNameInputStep
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemRecipientValidation

class NameServiceMock : GemNameServiceInterface {
    val requests = mutableListOf<Pair<String, Chain>>()

    override suspend fun getNameRecord(name: String, chain: uniffi.gemstone.Chain): GemNameRecordState {
        requests.add(name to Chain.entries.first { it.string == chain })
        return GemNameRecordState.Complete(mockNameRecord().toGem())
    }

    override fun isNameSupported(name: String): Boolean = name.split(".").size >= 2

    override fun nameInputStep(state: GemNameRecordState, name: String, hasChain: Boolean): GemNameInputStep = when {
        name.isEmpty() -> GemNameInputStep.Reset
        state is GemNameRecordState.Loading && state.name == name -> GemNameInputStep.Unchanged
        state is GemNameRecordState.Complete && state.record.name == name -> GemNameInputStep.Unchanged
        !hasChain || !isNameSupported(name) -> GemNameInputStep.Reset
        else -> GemNameInputStep.Resolve(name, 500u)
    }

    override fun recipient(chain: uniffi.gemstone.Chain, input: String, state: GemNameRecordState, memo: String?, references: List<String>): GemRecipient =
        GemRecipient(address = input, memo = memo, references = references)

    override fun resolvedState(state: GemNameRecordState, name: String, resolved: GemNameRecordState): GemNameRecordState =
        if (state is GemNameRecordState.Loading && state.name == name) resolved else state

    override fun validateRecipient(chain: uniffi.gemstone.Chain, input: String, state: GemNameRecordState): GemRecipientValidation =
        GemRecipientValidation(isValid = true, address = input, showsError = false)
}
