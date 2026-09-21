package com.gemwallet.android.ui.models.name

import com.gemwallet.android.ext.getNameRecord
import com.gemwallet.android.ext.toGem
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
import uniffi.gemstone.GemNameInputStep
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface

class NameRecordController(private val nameService: GemNameServiceInterface, private val scope: CoroutineScope) {
    private var job: Job? = null
    private val _state = MutableStateFlow<GemNameRecordState>(GemNameRecordState.None)
    val state: StateFlow<GemNameRecordState> = _state.asStateFlow()

    fun getNameRecord(value: String, chain: Chain?) {
        when (val step = nameService.nameInputStep(_state.value, value, chain?.toGem())) {
            GemNameInputStep.Unchanged -> return
            GemNameInputStep.Reset -> reset()
            is GemNameInputStep.Resolve -> loadNameRecord(step, requireNotNull(chain))
        }
    }

    private fun loadNameRecord(step: GemNameInputStep.Resolve, chain: Chain) {
        job?.cancel()
        _state.value = GemNameRecordState.Loading(step.name, chain.toGem())
        job = scope.launch {
            delay(step.debounceMilliseconds.toLong())
            val resolved = try {
                nameService.getNameRecord(step.name, chain)
            } catch (e: CancellationException) {
                throw e
            } catch (_: Throwable) {
                GemNameRecordState.Error
            }
            ensureActive()
            _state.value = nameService.resolvedState(_state.value, step.name, chain.toGem(), resolved)
        }
    }

    fun reset() {
        job?.cancel()
        _state.value = GemNameRecordState.None
    }
}
