package com.gemwallet.android.ui.models.name

import com.gemwallet.android.ext.getNameRecord
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
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface

class NameRecordController(
    private val nameService: GemNameServiceInterface,
    private val scope: CoroutineScope,
) {
    private var job: Job? = null
    private val _state = MutableStateFlow<GemNameRecordState>(GemNameRecordState.None)
    val state: StateFlow<GemNameRecordState> = _state.asStateFlow()

    fun getNameRecord(value: String, chain: Chain?) {
        if (value.isEmpty()) {
            reset()
            return
        }
        if (value == _state.value.requestedName()) {
            return
        }
        loadNameRecord(value, chain)
    }

    private fun loadNameRecord(input: String, chain: Chain?) {
        job?.cancel()
        _state.value = GemNameRecordState.None
        if (chain == null || !nameService.isNameSupported(input)) {
            return
        }
        _state.value = GemNameRecordState.Loading(input)
        job = scope.launch {
            delay(nameService.nameRecordDebounceMilliseconds().toLong())
            val resolved = try {
                nameService.getNameRecord(input, chain)
            } catch (e: CancellationException) {
                throw e
            } catch (_: Throwable) {
                GemNameRecordState.Error
            }
            ensureActive()
            _state.value = resolved
        }
    }

    fun reset() {
        job?.cancel()
        _state.value = GemNameRecordState.None
    }
}
