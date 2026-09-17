package com.gemwallet.android.ui.models.name

import com.gemwallet.android.ext.display
import com.gemwallet.android.ext.validateRecipient
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemRecipientErrorDisplay
import uniffi.gemstone.GemRecipientException
import uniffi.gemstone.GemRecipientValidation

class AddressInputModel(
    private val nameService: GemNameServiceInterface,
    scope: CoroutineScope,
    initialChain: Chain? = null,
) {
    private val nameRecordController = NameRecordController(nameService, scope)
    private val _text = MutableStateFlow("")
    private val _error = MutableStateFlow<GemRecipientErrorDisplay?>(null)
    private val _chain = MutableStateFlow(initialChain)

    val chain: Chain? get() = _chain.value

    val text: StateFlow<String> = _text.asStateFlow()
    val nameResolveState: StateFlow<GemNameRecordState> = nameRecordController.state
    val error: StateFlow<GemRecipientErrorDisplay?> = _error.asStateFlow()

    val isValid: StateFlow<Boolean> = combine(_text, nameRecordController.state, _chain) { text, resolve, chain ->
        isValid(text, resolve, chain)
    }.stateIn(scope, SharingStarted.Eagerly, false)

    val nameRecordState: GemNameRecordState get() = nameRecordController.state.value

    val resolvedAddress: String
        get() = chain?.let { validation(_text.value, nameRecordState, it).address } ?: _text.value

    fun onTextChange(value: String) {
        _text.value = value
        _error.value = null
        nameRecordController.getNameRecord(value, chain)
    }

    fun setChain(chain: Chain) {
        if (_chain.value == chain) return
        _chain.value = chain
        nameRecordController.reset()
        nameRecordController.getNameRecord(_text.value, chain)
        validate()
    }

    fun setScannedAddress(address: String) {
        _text.value = address
        nameRecordController.getNameRecord(address, chain)
        validate()
    }

    fun validate(): Boolean {
        val text = _text.value
        val chain = _chain.value
        val resolve = nameRecordController.state.value
        val valid = isValid(text, resolve, chain)
        _error.value = chain?.let { validation(text, resolve, it).error }
        return valid
    }

    fun markInvalid(rejection: GemRecipientException) {
        _error.value = _chain.value?.let { rejection.display(it) }
    }

    fun reset() {
        nameRecordController.reset()
        _text.value = ""
        _error.value = null
    }

    private fun isValid(text: String, resolve: GemNameRecordState, chain: Chain?): Boolean =
        chain != null && validation(text, resolve, chain).isValid

    private fun validation(text: String, state: GemNameRecordState, chain: Chain): GemRecipientValidation =
        nameService.validateRecipient(chain, text, state)
}
