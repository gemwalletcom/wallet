package com.gemwallet.android.features.settings.networks.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemServiceStatusInterface
import javax.inject.Inject

@HiltViewModel
class ServiceStatusViewModel @Inject constructor(private val service: GemServiceStatusInterface) : ViewModel() {
    private val session = MutableStateFlow(service.newSession())
    val sections = session.map { it.sections() }.stateIn(viewModelScope, SharingStarted.Eagerly, session.value.sections())
    private var fetchJob: Job? = null

    fun fetch() {
        fetchJob?.cancel()
        session.value = service.newSession()
        fetchJob = viewModelScope.launch {
            session.value.targets().forEach { target ->
                launch {
                    val status = service.status(target)
                    ensureActive()
                    session.update { it.onStatus(target, status) }
                }
            }
        }
    }
}
