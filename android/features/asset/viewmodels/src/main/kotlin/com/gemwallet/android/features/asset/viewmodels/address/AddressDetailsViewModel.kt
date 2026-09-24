package com.gemwallet.android.features.asset.viewmodels.address

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.wallet.core.primitives.ChainAddress
import dagger.assisted.Assisted
import dagger.assisted.AssistedFactory
import dagger.assisted.AssistedInject
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAddressDetailsServiceInterface
import uniffi.gemstone.GemAddressStore
import uniffi.gemstone.GemListSection

@HiltViewModel(assistedFactory = AddressDetailsViewModel.Factory::class)
class AddressDetailsViewModel @AssistedInject constructor(
    @Assisted private val chainAddress: ChainAddress,
    private val service: GemAddressDetailsServiceInterface,
    private val addressStore: GemAddressStore,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val details = MutableStateFlow(service.details(chainAddress.chain.string, chainAddress.address))

    private val addressName = flow { emit(addressStore.getAddressName(chainAddress.chain.string, chainAddress.address)) }.flowOn(ioDispatcher)

    val sections: StateFlow<List<GemListSection>> = combine(details, addressName) { details, addressName -> details.sections(addressName) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    suspend fun refresh() = withContext(ioDispatcher) {
        details.value = service.refresh(details.value)
    }

    @AssistedFactory
    interface Factory {
        fun create(chainAddress: ChainAddress): AddressDetailsViewModel
    }
}
