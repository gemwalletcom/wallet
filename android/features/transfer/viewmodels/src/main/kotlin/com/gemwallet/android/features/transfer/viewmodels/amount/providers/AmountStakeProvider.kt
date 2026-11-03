package com.gemwallet.android.features.transfer.viewmodels.amount.providers

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AmountParams
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.Resource
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import uniffi.gemstone.GemAmountRequest
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemStakeValidatorOptions

class AmountStakeProvider(val params: AmountParams.Stake, validators: Flow<List<DelegationValidator>>, private val service: GemAmountServiceInterface, scope: CoroutineScope, ioDispatcher: CoroutineDispatcher) {

    private val chain = params.assetId.chain.string

    private val stakeInput = MutableStateFlow(params.input)

    val validatorOptions: StateFlow<GemStakeValidatorOptions?> =
        combine(stakeInput, validators) { input, validators -> service.stakeValidatorOptions(chain, input, validators.map { it.toGem() }) }
            .flowOn(ioDispatcher)
            .stateIn(scope, SharingStarted.Eagerly, null)

    fun selectValidator(id: String?) {
        val options = validatorOptions.value ?: return
        val validator = options.sections.flatMap { it.rows }.firstOrNull { it.validator.id == id }?.validator ?: return
        stakeInput.update { it.withValidator(validator) }
    }

    fun setResource(value: Resource) {
        stakeInput.update { it.withResource(value.toGem()) }
    }

    val request: StateFlow<GemAmountRequest?> = stakeInput
        .map { GemAmountRequest.Stake(it) }
        .stateIn(scope, SharingStarted.Eagerly, null)
}
