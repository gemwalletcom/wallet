package com.gemwallet.android.features.transfer.viewmodels.amount.providers

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.AssetId
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
import uniffi.gemstone.GemGroupList
import uniffi.gemstone.GemListItem
import uniffi.gemstone.GemStakeAmountInput
import uniffi.gemstone.GemStakeServiceInterface

class AmountStakeProvider(assetId: AssetId, input: GemStakeAmountInput, validators: Flow<List<DelegationValidator>>, private val stakeService: GemStakeServiceInterface, scope: CoroutineScope, ioDispatcher: CoroutineDispatcher) {

    private val chain = assetId.chain.string

    private val stakeInput = MutableStateFlow(input)

    val validatorOptions: StateFlow<GemGroupList?> =
        combine(stakeInput, validators) { input, validators -> stakeService.stakeValidatorOptions(chain, input, validators.map { it.toGem() }) }
            .flowOn(ioDispatcher)
            .stateIn(scope, SharingStarted.Eagerly, null)

    fun selectValidator(id: String?) {
        val options = validatorOptions.value ?: return
        val validator = options.groups.flatMap { it.items }.filterIsInstance<GemListItem.Validator>().firstOrNull { it.row.validator.id == id }?.row?.validator ?: return
        stakeInput.update { it.withValidator(validator) }
    }

    fun setResource(value: Resource) {
        stakeInput.update { it.withResource(value.toGem()) }
    }

    val request: StateFlow<GemAmountRequest?> = stakeInput
        .map { GemAmountRequest.Stake(it) }
        .stateIn(scope, SharingStarted.Eagerly, null)
}
