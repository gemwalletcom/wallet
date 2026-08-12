package com.gemwallet.android.ui.models.name

import com.gemwallet.android.cases.name.ResolveName
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

class NameResolveController(
    private val resolveName: ResolveName,
    private val scope: CoroutineScope,
) {
    private var job: Job? = null
    private val _state = MutableStateFlow<NameRecordState>(NameRecordState.None)
    val state: StateFlow<NameRecordState> = _state.asStateFlow()

    fun resolve(value: String, chain: Chain?) {
        if (value.isEmpty()) {
            reset()
            return
        }
        if (value == _state.value.nameRecord?.name) {
            return
        }
        startResolve(value, chain)
    }

    private fun startResolve(input: String, chain: Chain?) {
        job?.cancel()
        _state.value = NameRecordState.None
        if (chain == null || !resolveName.canResolveName(input)) {
            return
        }
        _state.value = NameRecordState.Loading
        job = scope.launch {
            delay(DEBOUNCE_MS)
            val record = try {
                resolveName.resolveName(input, chain)
            } catch (e: CancellationException) {
                throw e
            } catch (_: Throwable) {
                null
            }
            ensureActive()
            val resolved = record?.takeIf { it.address.isNotEmpty() && it.name.isNotEmpty() }
            _state.value = when {
                resolved != null -> NameRecordState.Complete(resolved)
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
