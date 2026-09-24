package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.transfer_amount.viewmodels.models.AmountExtrasUIModel
import com.gemwallet.android.features.transfer_amount.viewmodels.models.ValidatorsUIModel
import com.gemwallet.android.features.transfer_amount.viewmodels.models.uiModel
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.ui.components.list_item.uiModel
import com.wallet.core.primitives.Resource
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemStakeAmountInput
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemStakeValidatorSelection
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemValidatorRow

class AmountStakeProvider(val params: AmountParams.Stake, getAssetInfo: GetAssetInfo, private val stakeService: GemStakeServiceInterface, scope: CoroutineScope) : AmountDataProvider(scope) {

    override val assetInfo: StateFlow<AssetInfo?> =
        getAssetInfo(params.assetId)
            .flowOn(Dispatchers.IO)
            .stateIn(scope, SharingStarted.Eagerly, null)

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

    val validatorRows: StateFlow<ValidatorsUIModel?> = validatorSelection
        .map { it?.uiModel() }
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

    override val extras: StateFlow<AmountExtrasUIModel> = when (params.input) {
        is GemStakeAmountInput.Freeze, is GemStakeAmountInput.Unfreeze -> selectedResource.map { AmountExtrasUIModel.Resources(resourceOptions, it) }

        else -> combine(validatorState, canSelectValidator) { row, canSelect ->
            row?.let { AmountExtrasUIModel.Validator(it.uiModel(), canSelect) } ?: AmountExtrasUIModel.None
        }
    }.stateIn(scope, SharingStarted.Eagerly, AmountExtrasUIModel.None)

    override val amountType: StateFlow<GemAmountType?> =
        combine(selected, selectedResource) { current, resource -> current?.confirmed(resource)?.amountType() }
            .flowOn(Dispatchers.IO)
            .stateIn(scope, SharingStarted.Eagerly, null)

    override suspend fun buildTransfer(amount: Crypto, isMax: Boolean): GemTransferData {
        val current = assetInfo.filterNotNull().first()
        val confirmed = checkNotNull(selected.value?.confirmed(selectedResource.value)) { "stake action requires a selection" }
        return stakeService.stakeTransferData(current.asset.toGem(), confirmed.stakeType(), amount.atomicValue, isMax)
    }
}
