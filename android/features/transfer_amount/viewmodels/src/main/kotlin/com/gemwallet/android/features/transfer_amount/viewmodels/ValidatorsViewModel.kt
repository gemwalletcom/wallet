package com.gemwallet.android.features.transfer_amount.viewmodels

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.stake.cases.GetDelegations
import uniffi.gemstone.GemStakeServiceInterface
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.features.transfer_amount.models.ValidatorsSource
import com.gemwallet.android.features.transfer_amount.models.ValidatorsUIState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ValidatorsViewModel @Inject constructor(
    private val getValidators: GetValidators,
    private val getDelegations: GetDelegations,
    private val service: GemStakeServiceInterface,
    val savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val source = MutableStateFlow<ValidatorsSource?>(null)

    val validators = source.filterNotNull()
        .flatMapLatest { source ->
            when (source) {
                is ValidatorsSource.ChainValidators -> getValidators(source.assetId)
                is ValidatorsSource.Rewards -> getDelegations(source.walletId, source.assetId)
                    .map { delegations ->
                        delegations
                            .filter { it.base.rewards > BigInteger.ZERO }
                            .map { it.validator }
                    }
            }
        }
        .stateIn(viewModelScope, SharingStarted.Companion.Eagerly, emptyList())

    val uiState = combine(source, validators) { source, validators ->
        when {
            source == null -> ValidatorsUIState.Loading
            validators.isNotEmpty() -> {
                val recommended = when (source) {
                    is ValidatorsSource.ChainValidators -> service.recommendedValidators(source.assetId.chain.string, validators.map { it.toGem() }).map { it.toPrimitives() }
                    is ValidatorsSource.Rewards -> emptyList()
                }
                ValidatorsUIState.Loaded(
                    loading = false,
                    recommended = recommended,
                    validators = validators,
                )
            }

            else -> ValidatorsUIState.Empty
        }
    }.stateIn(viewModelScope, SharingStarted.Companion.Eagerly, ValidatorsUIState.Loading)

    fun init(source: ValidatorsSource) {
        this.source.update { source }
    }
}
