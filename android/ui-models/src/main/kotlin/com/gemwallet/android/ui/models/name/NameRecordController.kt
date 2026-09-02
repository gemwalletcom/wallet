package com.gemwallet.android.ui.models.name

import com.gemwallet.android.domains.name.AddressInputResolving
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class NameRecordController(
    private val resolving: AddressInputResolving,
    private val scope: CoroutineScope,
) {
    private var job: Job? = null
    private val _state = MutableStateFlow<NameRecordState>(NameRecordState.None)
    val state: StateFlow<NameRecordState> = _state.asStateFlow()

    fun getNameRecord(value: String, chain: Chain?) {
        if (value.isEmpty()) {
            reset()
            return
        }
        if (value == _state.value.nameRecord?.name) {
            return
        }
        loadNameRecord(value, chain)
    }

    private fun loadNameRecord(input: String, chain: Chain?) {
        job?.cancel()
        _state.value = NameRecordState.None
        if (chain == null || !resolving.isNameSupported(input)) {
            return
        }
        _state.value = NameRecordState.Loading
        job = scope.launch {
            delay(DEBOUNCE_MS)
            val record = try {
                resolving.getNameRecord(input, chain)
            } catch (e: CancellationException) {
                throw e
            } catch (_: Throwable) {
                null
            }
            ensureActive()
            val nameRecord = record?.takeIf { it.address.isNotEmpty() && it.name.isNotEmpty() }
            _state.value = when {
                nameRecord != null -> NameRecordState.Complete(nameRecord)
                input.isNotEmpty() -> NameRecordState.Error
                else -> NameRecordState.None
            }
        }
    }

    fun reset() {
        job?.cancel()
        _state.value = NameRecordState.None
    }

    private companion object {
        const val DEBOUNCE_MS = 500L
    }
}
