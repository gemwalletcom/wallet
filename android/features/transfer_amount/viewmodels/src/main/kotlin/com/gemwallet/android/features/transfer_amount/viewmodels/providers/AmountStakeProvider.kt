package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.transfer_amount.viewmodels.models.AmountExtrasUIModel
import com.gemwallet.android.model.AmountParams
import com.wallet.core.primitives.Resource
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import uniffi.gemstone.GemAmountRequest
import uniffi.gemstone.GemStakeAmountInput
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemStakeValidatorSelection
import uniffi.gemstone.GemValidatorRow

class AmountStakeProvider(val params: AmountParams.Stake, private val stakeService: GemStakeServiceInterface, scope: CoroutineScope) {

    private val stakeInput = MutableStateFlow(params.input)

    private val selectedResource = MutableStateFlow(
        when (val input = params.input) {
            is GemStakeAmountInput.Freeze -> input.resource.toPrimitives()
            is GemStakeAmountInput.Unfreeze -> input.resource.toPrimitives()
            else -> Resource.Bandwidth
        },
    )
    val resource: StateFlow<Resource> = selectedResource.asStateFlow()

    val resourceOptions: List<Resource> by lazy { stakeService.resourceOptions(params.assetId.chain.string).map { it.toPrimitives() } }

    fun setResource(value: Resource) {
        selectedResource.update { value }
    }

    private val selected: StateFlow<StakeSelection?> = stakeInput
        .map { current -> StakeSelection(current, stakeService.stakeValidatorSelection(params.assetId.chain.string, current)) }
        .flowOn(Dispatchers.IO)
        .stateIn(scope, SharingStarted.Eagerly, null)

    val validatorSelection: StateFlow<GemStakeValidatorSelection?> = selected
        .map { it?.validators }
        .stateIn(scope, SharingStarted.Eagerly, null)

    val validatorState: StateFlow<GemValidatorRow?> = selected
        .map { it?.validators?.validator }
        .stateIn(scope, SharingStarted.Eagerly, null)

    private data class StakeSelection(val input: GemStakeAmountInput, val validators: GemStakeValidatorSelection) {
        fun confirmed(resource: Resource): GemStakeAmountInput = (validators.validator?.validator?.let(input::withValidator) ?: input).withResource(resource.toGem())
    }

    val canSelectValidator: StateFlow<Boolean> = validatorSelection
        .map { it?.canSelect == true }
        .stateIn(scope, SharingStarted.Eagerly, false)

    fun selectValidator(id: String?) {
        val selection = validatorSelection.value ?: return
        val validator = (selection.recommended + selection.options).firstOrNull { it.validator.id == id }?.validator ?: return
        stakeInput.update { it.withValidator(validator) }
    }

    val extras: StateFlow<AmountExtrasUIModel> = when (params.input) {
        is GemStakeAmountInput.Freeze, is GemStakeAmountInput.Unfreeze -> selectedResource.map { AmountExtrasUIModel.Resources(resourceOptions, it) }

        else -> combine(validatorState, canSelectValidator) { row, canSelect ->
            row?.let { AmountExtrasUIModel.Validator(it, canSelect) } ?: AmountExtrasUIModel.None
        }
    }.stateIn(scope, SharingStarted.Eagerly, AmountExtrasUIModel.None)

    val request: StateFlow<GemAmountRequest?> =
        combine(selected, selectedResource) { current, resource -> current?.confirmed(resource)?.let { GemAmountRequest.Stake(it) } }
            .stateIn(scope, SharingStarted.Eagerly, null)
}
