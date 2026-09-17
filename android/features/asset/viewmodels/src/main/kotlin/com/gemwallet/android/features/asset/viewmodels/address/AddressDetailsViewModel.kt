package com.gemwallet.android.features.asset.viewmodels.address

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import com.wallet.core.primitives.ChainAddress
import dagger.assisted.Assisted
import dagger.assisted.AssistedFactory
import dagger.assisted.AssistedInject
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAddressDetails
import uniffi.gemstone.GemListSection
import uniffi.gemstone.GemAddressDetailsServiceInterface

@HiltViewModel(assistedFactory = AddressDetailsViewModel.Factory::class)
class AddressDetailsViewModel @AssistedInject constructor(
    @Assisted private val chainAddress: ChainAddress,
    private val service: GemAddressDetailsServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val details = MutableStateFlow(service.details(chainAddress.chain.string, chainAddress.address))

    val sections: StateFlow<List<GemListSection>> = details
        .map { details -> details.sections() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, details.value.sections())

    suspend fun refresh() = withContext(ioDispatcher) {
        details.value = service.refresh(details.value)
    }

    @AssistedFactory
    interface Factory {
        fun create(chainAddress: ChainAddress): AddressDetailsViewModel
    }
}
