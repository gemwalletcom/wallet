package com.gemwallet.android.ui.models.name

import com.gemwallet.android.cases.name.ResolveName
import com.gemwallet.android.ext.checksumAddress
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn

class AddressInputModel(
    private val resolveName: ResolveName,
    scope: CoroutineScope,
    private val validateAddress: (address: String, chain: Chain) -> Boolean,
    initialChain: Chain? = null,
) {
    private val resolver = NameResolveController(resolveName, scope)
    private val _text = MutableStateFlow("")
    private val _showError = MutableStateFlow(false)
    private val _chain = MutableStateFlow(initialChain)

    val chain: Chain? get() = _chain.value

    val text: StateFlow<String> = _text.asStateFlow()
    val nameResolveState: StateFlow<NameRecordState> = resolver.state
    val showError: StateFlow<Boolean> = _showError.asStateFlow()

    val isValid: StateFlow<Boolean> = combine(_text, resolver.state, _chain) { text, resolve, chain ->
        isValid(text, resolve, chain)
    }.stateIn(scope, SharingStarted.Eagerly, false)

    val nameRecord get() = resolver.state.value.nameRecord

    val resolvedAddress: String
        get() {
            val address = nameRecord?.address?.takeIf { it.isNotEmpty() } ?: _text.value
            return chain?.checksumAddress(address) ?: address
        }

    fun onTextChange(value: String) {
        _text.value = value
        _showError.value = false
        resolver.resolve(value, chain)
    }

    fun setChain(chain: Chain) {
        if (_chain.value == chain) return
        _chain.value = chain
        resolver.reset()
        resolver.resolve(_text.value, chain)
        validate()
    }

    fun applyExternalAddress(address: String) {
        _text.value = address
        resolver.resolve(address, chain)
        validate()
    }

    fun validate(): Boolean {
        val text = _text.value
        val valid = isValid(text, resolver.state.value, _chain.value)
        _showError.value = text.isNotBlank() && !resolveName.canResolveName(text) && !valid
        return valid
    }

    fun markInvalid() {
        _showError.value = _text.value.isNotBlank()
    }

    fun reset() {
        resolver.reset()
        _text.value = ""
        _showError.value = false
    }

    private fun isValid(text: String, resolve: NameRecordState, chain: Chain?): Boolean = when (resolve) {
        NameRecordState.Loading, NameRecordState.Error -> false
        is NameRecordState.Complete -> true
        NameRecordState.None -> text.isNotBlank() && chain != null && validateAddress(chain.checksumAddress(text), chain)
    }
}
