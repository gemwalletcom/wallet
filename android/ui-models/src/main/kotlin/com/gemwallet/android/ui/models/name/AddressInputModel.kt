package com.gemwallet.android.ui.models.name

import com.gemwallet.android.application.recipient.cases.GetNameRecord
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemRecipientValidation

class AddressInputModel(
    private val getNameRecord: GetNameRecord,
    private val nameService: GemNameServiceInterface,
    scope: CoroutineScope,
    initialChain: Chain? = null,
) {
    private val nameRecordController = NameRecordController(getNameRecord, scope)
    private val _text = MutableStateFlow("")
    private val _showError = MutableStateFlow(false)
    private val _chain = MutableStateFlow(initialChain)

    val chain: Chain? get() = _chain.value

    val text: StateFlow<String> = _text.asStateFlow()
    val nameResolveState: StateFlow<NameRecordState> = nameRecordController.state
    val showError: StateFlow<Boolean> = _showError.asStateFlow()

    val isValid: StateFlow<Boolean> = combine(_text, nameRecordController.state, _chain) { text, resolve, chain ->
        isValid(text, resolve, chain)
    }.stateIn(scope, SharingStarted.Eagerly, false)

    val nameRecord get() = nameRecordController.state.value.nameRecord

    val resolvedAddress: String
        get() = chain?.let { validation(_text.value, nameRecord, it).address } ?: _text.value

    fun onTextChange(value: String) {
        _text.value = value
        _showError.value = false
        nameRecordController.getNameRecord(value, chain)
    }

    fun setChain(chain: Chain) {
        if (_chain.value == chain) return
        _chain.value = chain
        nameRecordController.reset()
        nameRecordController.getNameRecord(_text.value, chain)
        validate()
    }

    fun applyExternalAddress(address: String) {
        _text.value = address
        nameRecordController.getNameRecord(address, chain)
        validate()
    }

    fun validate(): Boolean {
        val text = _text.value
        val chain = _chain.value
        val resolve = nameRecordController.state.value
        val valid = isValid(text, resolve, chain)
        _showError.value = if (chain == null) text.isNotBlank() else validation(text, resolve.nameRecord, chain).showsError
        return valid
    }

    fun markInvalid() {
        _showError.value = _text.value.isNotBlank()
    }

    fun reset() {
        nameRecordController.reset()
        _text.value = ""
        _showError.value = false
    }

    private fun isValid(text: String, resolve: NameRecordState, chain: Chain?): Boolean = when (resolve) {
        NameRecordState.Loading, NameRecordState.Error -> false
        is NameRecordState.Complete, NameRecordState.None -> chain != null && validation(text, resolve.nameRecord, chain).isValid
    }

    private fun validation(text: String, nameRecord: NameRecord?, chain: Chain): GemRecipientValidation =
        nameService.validateRecipient(chain.string, text, nameRecord?.toJson())
}
