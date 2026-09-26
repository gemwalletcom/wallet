package com.gemwallet.android.features.transfer.viewmodels.amount.providers

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.transfer.viewmodels.amount.models.AmountExtrasUIModel
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
import uniffi.gemstone.GemStakeAmountSelection
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemStakeValidatorOptions

class AmountStakeProvider(val params: AmountParams.Stake, validators: Flow<List<DelegationValidator>>, private val stakeService: GemStakeServiceInterface, scope: CoroutineScope, ioDispatcher: CoroutineDispatcher) {

    private val chain = params.assetId.chain.string

    private val stakeInput = MutableStateFlow(params.input)

    private val selection: StateFlow<GemStakeAmountSelection?> = stakeInput
        .map { stakeService.stakeAmountSelection(chain, it) }
        .stateIn(scope, SharingStarted.Eagerly, null)

    val validatorOptions: StateFlow<GemStakeValidatorOptions?> =
        combine(stakeInput, validators) { input, validators -> stakeService.stakeValidatorOptions(chain, input, validators.map { it.toGem() }) }
            .flowOn(ioDispatcher)
            .stateIn(scope, SharingStarted.Eagerly, null)

    val selectedValidatorId: StateFlow<String?> = selection
        .map { (it as? GemStakeAmountSelection.Validator)?.validator?.validator?.id }
        .stateIn(scope, SharingStarted.Eagerly, null)

    fun selectValidator(id: String?) {
        val options = validatorOptions.value ?: return
        val validator = (options.recommended + options.options).firstOrNull { it.validator.id == id }?.validator ?: return
        stakeInput.update { it.withValidator(validator) }
    }

    fun setResource(value: Resource) {
        stakeInput.update { it.withResource(value.toGem()) }
    }

    val extras: StateFlow<AmountExtrasUIModel> = selection
        .map { selection ->
            when (selection) {
                null -> AmountExtrasUIModel.None
                is GemStakeAmountSelection.Validator -> AmountExtrasUIModel.Validator(selection.validator, selection.canSelect)
                is GemStakeAmountSelection.Resource -> AmountExtrasUIModel.Resources(selection.options.map { it.toPrimitives() }, selection.selected.toPrimitives())
            }
        }
        .stateIn(scope, SharingStarted.Eagerly, AmountExtrasUIModel.None)

    val request: StateFlow<GemAmountRequest?> = stakeInput
        .map { GemAmountRequest.Stake(it) }
        .stateIn(scope, SharingStarted.Eagerly, null)
}
